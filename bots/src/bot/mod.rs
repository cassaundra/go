use goban::rules::*;
use goban::rules::game::*;

mod random;
pub use random::*;

mod gtp;
pub use gtp::*;

pub const RECURSE_DEPTH: usize = 1;

pub trait Bot {
    fn play(&mut self, game: &Game) -> Move;
}

pub trait DefaultingBot<'a, B: Bot + 'a> {
    fn play(&mut self, game: &Game) -> Option<Move>;
    fn default(&mut self) -> &'a mut B;
}

impl<'a, B: Bot + 'a> Bot for dyn DefaultingBot<'a, B> {
    fn play(&mut self, game: &Game) -> Move {
        match DefaultingBot::play(self, game) {
            Some(mov) => mov,
            None => self.default().play(game),
        }
    }
}
