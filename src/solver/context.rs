use std::cell::RefCell;

use crate::config::Config;
use crate::models::Problem;
use crate::utils::Random;

#[derive(Debug)]
pub struct Context {
    pub problem: Problem,
    pub config: RefCell<Config>,
    pub random: Random,
}

impl Context {
    pub fn new(problem: Problem, config: Config) -> Self {
        let random = if config.deterministic {
            log::info!("Deterministic with seed: {}", config.seed);
            Random::from_seed(config.seed)
        } else {
            Random::new()
        };
        Self {
            problem,
            config: RefCell::new(config),
            random,
        }
    }
}
