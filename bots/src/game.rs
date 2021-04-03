use goban::rules::*;
use goban::rules::game::*;

use log::*;

use crate::Bot;

// TODO tournament stuff
// it doesn't really matter until bots are implemented anyway
// individual pairings are more interesting

// pub struct Tournament {
//     participants: Vec<Box<dyn Bot>>,
//     options: TournamentOptions,
// }

// TODO implement builder

#[derive(Copy, Clone, Debug)]
pub struct TournamentOptions {
    pub game_options: GameOptions,
    pub num_matches: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct GameOptions {
    pub rules: Rule,
    pub board_size: (u32, u32),
    pub max_moves: usize,
}

impl Default for GameOptions {
    fn default() -> Self {
        GameOptions {
            rules: JAPANESE,
            board_size: (19, 19),
            max_moves: 512,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PairingResults {
    pub black_wins: usize,
    pub white_wins: usize,
    pub draws: usize,
}

impl PairingResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_games(&self) -> usize {
        self.black_wins + self.white_wins + self.draws
    }

    pub fn percent_black_wins(&self) -> Option<f32> {
        self.percent(self.black_wins)
    }

    pub fn percent_white_wins(&self) -> Option<f32> {
        self.percent(self.white_wins)
    }

    pub fn percent_draws(&self) -> Option<f32> {
        self.percent(self.draws)
    }

    fn percent(&self, value: usize) -> Option<f32> {
        let total = self.total_games();

        if total > 0 {
            Some(value as f32 / total as f32)
        } else {
            None
        }
    }
}

pub fn play_pairing(options: &TournamentOptions, black: &mut impl Bot, white: &mut impl Bot) -> PairingResults {
    let mut results = PairingResults::new();

    for _ in 0..options.num_matches {
        let result = play_game(&options.game_options, black, white);

        match result.get_winner() {
            Some(Player::Black) => results.black_wins += 1,
            Some(Player::White) => results.white_wins += 1,
            None => results.draws += 1,
        }
    }

    results
}

pub fn play_game(options: &GameOptions, black: &mut impl Bot, white: &mut impl Bot) -> EndGame {
    let mut game = Game::builder()
        .size(options.board_size)
        .rule(options.rules)
        .build().unwrap();

    let mut moves = 0;

    while !game.is_over() && moves < options.max_moves {
        let turn = game.turn();
        let bot_move = match game.turn() {
            Player::Black => black.play(&game),
            Player::White => white.play(&game),
        };

        trace!("PLAY: {:?} at {:2?} (Move {})", turn, bot_move, moves + 1);

        // TODO handle illegal plays with Game::try_play
        // info!("SGF: {}", crate::sgf::game_to_sgf(&game, "Black", "White"));
        match game.try_play(bot_move) {
            Ok(_) => {},
            Err(err) => {
                error!("Error: {:?}", err);
            }
        }

        moves += 1;
    }

    if game.is_over() {
        game.outcome().unwrap()
    } else {
        let (black_score, white_score) = game.calculate_score();

        if black_score > white_score {
            EndGame::WinnerByScore(Player::Black, black_score - white_score)
        } else {
            EndGame::WinnerByScore(Player::White, white_score - black_score)
        }
    }
}
