use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use err_derive::Error;

use goban::pieces::stones::{Color, Stone};

use ::gtp::Command as GtpCommand;
use ::gtp::Response as GtpResponse;

use log::*;

use super::*;

type GtpResult<T> = std::result::Result<T, GtpError>;

// pub const DEFAULT_TIMEOUT: usize = 20 * 1000;

#[derive(Debug, Error)]
enum GtpError {
    #[error(display = "I/O error: {}", _0)]
    Io(#[error(source)] io::Error),
    #[error(display = "Response error: {:?}", _0)]
    Response(#[error(from)] ::gtp::ResponseError),
    #[error(display = "Response parse error: {:?}", _0)]
    ResponseParse(#[error(from)] ::gtp::ResponseParseError),
}

pub struct GtpBot {
    child: Child,
    child_stdin: ChildStdin,
    child_stdout: ChildStdout,
    is_setup: bool,
    // board_hash: Option<u64>,
}

impl Drop for GtpBot {
    fn drop(&mut self) {
        // TODO is this necessary?
        self.child.kill().expect("Could not kill child process.");
    }
}

impl GtpBot {
    pub fn new(command: &str, args: &[&str]) -> Self {
        let mut child = Command::new(&command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        GtpBot {
            child_stdin: child.stdin.take().unwrap(),
            child_stdout: child.stdout.take().unwrap(),
            child,
            is_setup: false,
        }
    }

    pub fn gnugo(level: usize) -> Self {
        GtpBot::new("/usr/bin/gnugo", &["--mode", "gtp", "--level", &level.to_string()])
    }

    fn send_command(&mut self, command: &GtpCommand) -> GtpResult<GtpResponse> {
        // trace!("Sending command: {}", command.to_string().trim());
        writeln!(self.child_stdin, "{}", command.to_string())?;

        self.get_response()
    }

    fn get_response(&mut self) -> GtpResult<GtpResponse> {
        let mut reader = BufReader::new(&mut self.child_stdout);

        // TODO timeout? and attempt counting

        let mut out = String::new();
        reader.read_line(&mut out)?;
        reader.read_line(&mut String::new())?;

        let mut rp = ::gtp::ResponseParser::new();
        rp.feed(&out);
        rp.feed("\n");

        rp.get_response().map_err(Into::into)
    }

    fn clear_board(&mut self) -> GtpResult<()> {
        self.send_command(&GtpCommand::new("clear_board"))?;
        Ok(())
    }

    fn push_stone(&mut self, stone: &Stone) -> GtpResult<()> {
        let play_cmd = GtpCommand::new_with_args("play", |eb| {
            let c = &stone.coordinates;
            eb.mv(stone.color == Color::White, ((c.0 + 1).into(), (c.1 + 1).into()))
        });
        self.send_command(&play_cmd)?;

        Ok(())
    }

    fn setup(&mut self, game: &Game) -> GtpResult<()> {
        // ensure that the bot is reset
        self.clear_board()?;

        // set komi
        self.send_command(&GtpCommand::new_with_args("komi", |eb| eb.f(game.komi())))?;

        // set board size
        let (width, height) = game.goban().size();
        self.send_command(&GtpCommand::new_with_args("boardsize", |eb| {
            eb.i(width as u32).i(height as u32).list()
        }))?;

        // call play command for each stone on the board
        for stone in game.goban().get_stones() {
            self.push_stone(&stone)?;
        }

        self.is_setup = true;

        Ok(())
    }

    fn sync_state(&mut self, game: &Game) -> GtpResult<()> {
        if self.is_setup {
            if game.passes() == 0 {
                if let Some(last) = game.history().last() {
                    let difference = crate::sgf::goban_difference(last, game.goban());

                    for stone in difference {
                        self.push_stone(&stone)?;
                    }
                }
            }
        } else {
            self.setup(game)?;
        }

        Ok(())
    }

    fn generate_move(&mut self, game: &Game) -> GtpResult<Move> {
        // execute the generate move command
        let genmove_cmd = GtpCommand::new_with_args("genmove", |eb| {
            if game.turn() == Player::White {
                eb.w()
            } else {
                eb.b()
            }
        });

        let response = self.send_command(&genmove_cmd)?;

        // parse the response into a vertex entity
        let mut ep = ::gtp::EntityParser::new(&response.text());
        let res = ep.vertex().result().unwrap();

        let m = match res[0] {
            ::gtp::Entity::Vertex((h, v)) => {
                // PASS is parsed as vertex 0 0
                if h == 0 && v == 0 {
                    Move::Pass
                } else {
                    // play with corrected offset
                    Move::Play(h as u8 - 1, v as u8 - 1)
                }
            }
            _ => Move::Pass,
        };

        Ok(m)
    }

    fn play_move(&mut self, game: &Game) -> GtpResult<Move> {
        // ensure that the gtp process has the current game state
        self.sync_state(game)?;

        // generate a move
        self.generate_move(game)
    }
}

impl Bot for GtpBot {
    fn play(&mut self, game: &Game) -> Move {
        self.play_move(game).unwrap_or_else(|err| {
            warn!("An error occurred for GTP client: {:?}", err);
            Move::Pass
        })
    }
}
