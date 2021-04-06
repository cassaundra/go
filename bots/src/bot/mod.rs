use goban::rules::game::*;
use goban::rules::*;

mod random;
pub use random::*;

mod gtp;
pub use self::gtp::*;

// mod mcts;
// pub use mcts::*;

pub trait Bot {
    fn play(&mut self, game: &Game) -> Move;
}

pub trait DefaultingBot<'a, B: Bot + 'a> {
    fn play(&mut self, game: &Game) -> Option<Move>;
    fn default(&mut self) -> &'a mut B;
}

impl<'a, B: Bot + 'a> Bot for dyn DefaultingBot<'a, B> {
    fn play(&mut self, game: &Game) -> Move {
        self.play(game).unwrap_or_else(|| self.default().play(game))
    }
}
