use rand::seq::IteratorRandom;
use rand::{Rng, rngs::ThreadRng, thread_rng};

use super::*;

// TODO use seeded RNGs

#[derive(Debug)]
pub struct RandomBot {
    rng: ThreadRng,
}

impl RandomBot {
    pub fn new() -> Self {
        Self {
            rng: thread_rng(),
        }
    }
}

impl Bot for RandomBot {
    fn play(&mut self, game: &Game) -> Move {
        game.legals()
            .choose(&mut rand::thread_rng())
            .map(|point| Move::Play(point.0,point.1))
            .unwrap_or(Move::Pass)
    }
}

pub struct MixedBot {
    bots: Vec<Box<dyn Bot>>,
    weight_bounds: Vec<f32>,
    rng: ThreadRng,
}

impl MixedBot {
    pub fn new(bots: Vec<Box<dyn Bot>>) -> Self {
        let len = bots.len();
        MixedBot::new_weighted(bots, vec![1.; len])
    }

    pub fn new_weighted(bots: Vec<Box<dyn Bot>>, weights: Vec<f32>) -> Self {
        assert!(bots.len() > 1);
        assert_eq!(bots.len(), weights.len());

        // remap weights so that they sum to 1.0

        let sum: f32 = weights.iter().sum();

        assert!(sum > 0.);
        // TODO assert all weights are > 0
        // and length > 0

        let weight_bounds = weights.iter()
            .map(|w| w / sum)
            .scan(0., |sum_weight, weight| {
                *sum_weight += weight;
                Some(*sum_weight)
            })
            .collect();

        MixedBot {
            bots,
            weight_bounds: weight_bounds,
            rng: thread_rng(),
        }
    }
}

impl Bot for MixedBot {
    fn play(&mut self, game: &Game) -> Move {
        let r = self.rng.gen_range(0.0..1.0);

        let idx = self.weight_bounds.iter()
            .enumerate()
            .find_map(|(idx, &weight)| {
                if r < weight { Some(idx) } else { None }
            })
            .unwrap();

        self.bots[idx].play(game)
    }
}
