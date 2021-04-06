#[macro_use]
extern crate derive_builder;

extern crate log;

use log::*;

mod bot;
pub use bot::*;

mod game;
pub use game::*;

mod sgf;
pub use sgf::*;

fn main() {
    env_logger::init();
}
