//! Conversion from game states to the Smart Game Format (SGF).

use std::collections::HashSet;
use std::iter::repeat;

use goban::pieces::goban::Goban;
use goban::pieces::stones::{Color, Stone};
use goban::rules::game::*;
use goban::rules::*;

use sgf_parse::{serialize, SgfNodeBuilder, SgfProp};

/// Convert a game record to SGF format.
pub fn game_to_sgf(
    game: &Game, player_black: impl Into<String>, player_white: impl Into<String>,
) -> String {
    // root node
    let mut root = SgfNodeBuilder::new();

    // set game properties

    // black player
    root.properties.push(SgfProp::new(String::from("PB"), vec![player_black.into()]));
    // white player
    root.properties.push(SgfProp::new(String::from("PW"), vec![player_white.into()]));
    // komi
    root.properties.push(SgfProp::new(String::from("KM"), vec![game.komi().to_string()]));

    // game result
    if let Some(result) = game.outcome() {
        let formatted_result = match result {
            EndGame::WinnerByScore(player, score) => format!("{}+{}", player_letter(player), score),
            EndGame::WinnerByResign(player) => format!("{}+R", player),
            EndGame::WinnerByTime(player) => format!("{}+T", player),
            EndGame::WinnerByForfeit(player) => format!("{}+F", player),
            EndGame::Draw => "0".to_string(),
        };

        root.properties.push(SgfProp::new(String::from("RE"), vec![formatted_result]));
    }

    let history = game.history();

    let mut iter = history.iter().zip(history.iter().skip(1).chain(repeat(game.goban())));

    // call recursive function
    build_moves(&mut iter, &mut root);

    // finish by serializing node structure
    serialize(&root.build())
}

/// Transform board states into a recursive SGF move structure.
fn build_moves<'a>(
    iter: &mut impl Iterator<Item = (&'a Goban, &'a Goban)>, root: &mut SgfNodeBuilder,
) {
    if let Some((before, after)) = iter.next() {
        let mut node = SgfNodeBuilder::new();

        // push each new stone to this node

        let difference = goban_difference(&before, &after);

        for stone in difference {
            let player = match stone.color {
                Color::White => Player::White,
                Color::Black => Player::Black,
                Color::None => panic!("No color for stone."),
            };

            let coord = format!(
                "{}{}",
                ('a' as u8 + stone.coordinates.1) as char,
                ('a' as u8 + stone.coordinates.0) as char,
            );

            node.properties.push(SgfProp::new(player_letter(player), vec![coord]));
        }

        // set up the next board state
        build_moves(iter, &mut node);

        // push this node to the previous
        root.children.push(node);
    }
}

/// Convert a [Player](goban::rules::Player) into a SGF-friendly letter.
pub fn player_letter(player: Player) -> String {
    match player {
        Player::Black => String::from("B"),
        Player::White => String::from("W"),
    }
}

/// Calculate the difference in stones between two board states.
pub fn goban_difference(before: &Goban, after: &Goban) -> Vec<Stone> {
    let before: HashSet<Stone> = before.get_stones().collect();
    after.get_stones().filter(|stone| !before.contains(stone)).collect()
}
