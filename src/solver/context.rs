use std::cell::RefCell;
use std::time::Duration;

use crate::config::Config;
use crate::models::Problem;
use crate::solver::SearchHistory;
use crate::utils::Random;

#[derive(Debug)]
pub struct Context {
    pub problem: Problem,
    pub config: RefCell<Config>,
    pub random: Random,
    pub search_history: RefCell<SearchHistory>,
}

impl Context {
    pub fn new(problem: Problem, config: Config, search_history: SearchHistory) -> Self {
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
            search_history: RefCell::new(search_history),
        }
    }

    pub fn elapsed_as_secs(&self) -> u64 {
        self.elapsed().as_secs()
    }

    pub fn elapsed_as_secs_f64(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    pub fn elapsed(&self) -> Duration {
        self.search_history.borrow().start_time.elapsed()
    }

    pub fn terminate(&self) -> bool {
        self.elapsed_as_secs() >= self.config.borrow().time_limit
    }
}
