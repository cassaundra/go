extern crate log;

mod bot;
pub use bot::*;

mod game;
pub use game::*;

mod sgf;
pub use sgf::*;

fn main() {
    pretty_env_logger::init();
}
