use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use goban::pieces::stones::{Color, Stone};

use ::gtp::Command as GtpCommand;

use log::*;

use super::*;

pub const DEFAULT_TIMEOUT: usize = 20 * 1000;

pub struct GtpBot {
    child: Child,
    child_stdin: ChildStdin,
    child_stdout: ChildStdout,
    is_setup: bool,
}

impl Drop for GtpBot {
    fn drop(&mut self) {
        // TODO this is necessary, yes?
        self.child.kill()
            .expect("Could not kill child process.");
    }
}

impl GtpBot {
    pub fn new(command: String, args: Vec<String>) -> Self {
        let mut child = Command::new(&command)
            .args(&args)
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

    pub fn reset(&mut self) -> io::Result<()> {
        self.clear_board()?;
        self.is_setup = false;

        Ok(())
    }

    fn send_command(&mut self, command: &GtpCommand) -> io::Result<()> {
        // TODO is this cheap?
        // let mut writer = self.child.stdin.take().unwrap();
        writeln!(self.child_stdin, "{}", command.to_string())
    }

    fn clear_board(&mut self) -> io::Result<()> {
        self.send_command(&GtpCommand::new("clear_board"))
    }

    fn push_stone(&mut self, stone: &Stone) -> io::Result<()> {
        let play_cmd = GtpCommand::new_with_args("play", |eb| {
            let c = &stone.coordinates;
            eb.mv(stone.color == Color::White, ((c.0 + 1).into(), (c.1 + 1).into()))
        });
        self.send_command(&play_cmd)?;

        Ok(())
    }

    fn setup(&mut self, game: &Game) -> io::Result<()> {
        self.clear_board()?;

        // TODO set komi, board size

        // call play command for each stone on the board
        for stone in game.goban().get_stones() {
            self.push_stone(&stone)?;
        }

        self.is_setup = true;

        Ok(())
    }

    fn update(&mut self, game: &Game) -> std::io::Result<()> {
        let history = game.history();
        if history.len() > 1 {
            let difference = crate::sgf::goban_difference(history.last().unwrap(), game.goban());

            for stone in difference {
                self.push_stone(&stone)?;
            }
        }

        Ok(())
    }

    fn generate_move(&mut self, game: &Game) -> io::Result<Move> {
        let genmove_cmd = GtpCommand::new_with_args("genmove", |eb| {
            if game.turn() == goban::rules::Player::White {
                eb.w()
            } else {
                eb.b()
            }
        });
        self.send_command(&genmove_cmd)?;

        // let stdout = self.child.stdout.take().unwrap();
        let mut reader = BufReader::new(&mut self.child_stdout);

        let mut out = String::new();
        let mut attempts = 0;

        // TODO timeout
        // warn if resign by timeout

        while out.len() < 4 && attempts < 128 {
            out.clear();
            reader.read_line(&mut out)?;
            reader.read_line(&mut String::new())?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            attempts += 1;
        }

        let mut rp = ::gtp::ResponseParser::new();
        rp.feed(&format!("{}\n", out));

        let mut ep = ::gtp::EntityParser::new(&rp.get_response().unwrap().text());
        let res = ep.vertex().result().unwrap();

        let m = match res[0] {
            ::gtp::Entity::Vertex((h, v)) => {
                if h == 0 && v == 0 {
                    Move::Pass
                } else {
                    Move::Play(h as u8 - 1, v as u8 - 1)
                }
            }
            _ => Move::Pass,
        };

        Ok(m)
    }
}

impl Bot for GtpBot {
    fn play(&mut self, game: &Game) -> Move {
        if !self.is_setup {
            self.setup(game);
        }

        self.update(game);
        self.generate_move(game).expect("I/O error")
    }
}
