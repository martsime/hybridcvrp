use std::cell::RefCell;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::models::{MatrixProvider, Problem};
use crate::solver::SearchHistory;
use crate::utils::{ProblemParser, Random};

#[derive(Debug)]
pub struct Context {
    pub problem: Problem,
    pub matrix_provider: MatrixProvider,
    pub config: RefCell<Config>,
    pub random: Random,
    pub search_history: RefCell<SearchHistory>,
    pub iteration: RefCell<u64>,
}

impl Context {
    pub fn new(mut parser: ProblemParser, config: Config, start_time: Instant) -> Self {
        let problem = parser.problem.take().expect("Failed to parse problem");
        let random = if config.deterministic {
            log::info!("Deterministic with seed: {}", config.seed);
            Random::from_seed(config.seed)
        } else {
            Random::new()
        };

        let matrix_provider = MatrixProvider::new(&problem, &config, parser.matrix.take());
        log::info!("Matrices built!");

        let context = Self {
            problem,
            matrix_provider,
            config: RefCell::new(config),
            random,
            search_history: RefCell::new(SearchHistory::new(start_time)),
            iteration: RefCell::new(0),
        };

        context.setup();
        context
    }

    pub fn setup(&self) {
        self.config.borrow_mut().num_vehicles = self.initial_num_vehicles();
        self.reset_penalty();
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
        let config = self.config.borrow();
        self.elapsed_as_secs() >= config.time_limit
            || config.max_iterations.map_or(false, |max_iterations| {
                *self.iteration.borrow() >= max_iterations
            })
    }

    pub fn next_iteration(&self) {
        *self.iteration.borrow_mut() += 1;
    }

    pub fn reset_penalty(&self) {
        let max_distance = self.matrix_provider.distance.max();
        let max_demand = self.problem.max_demand();
        self.config.borrow_mut().penalty_capacity = Self::initial_penalty(max_distance, max_demand);
    }

    // Minimum number of vehicles from the LP bin packing problem
    pub fn vehicle_lower_bound(&self) -> u64 {
        let total_demand = self.problem.total_demand();
        let capacity = self.problem.vehicle.cap;
        (total_demand / capacity).ceil() as u64
    }

    fn initial_num_vehicles(&self) -> u64 {
        // Safety maring: 20% + 2 more than the bin packing lower bound
        let lower_bound = self.vehicle_lower_bound() as f64;
        (1.2 as f64 * lower_bound + 2.0).ceil() as u64
    }

    // Estimation of the initial penalty
    fn initial_penalty(max_distance: Option<f64>, max_demand: Option<f64>) -> f64 {
        match (max_distance, max_demand) {
            (Some(distance), Some(demand)) => 0.0001f64.max(10_000f64.min(distance / demand)),
            _ => 100.0,
        }
    }

    pub fn from_mapping(&self, mapping: &[usize]) -> Self {
        let mut search_history =
            SearchHistory::new(self.search_history.borrow().start_time.clone());
        search_history.log_new_best(false);

        Self {
            problem: self.problem.from_mapping(mapping),
            matrix_provider: self.matrix_provider.from_mapping(mapping),
            config: self.config.clone(),
            random: self.random.clone(),
            search_history: RefCell::new(search_history),
            iteration: RefCell::new(0),
        }
    }
}
