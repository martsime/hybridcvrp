use instant::Instant;
use wasm_bindgen::prelude::*;

use crate::config::Config;
use crate::models::{Coordinate, Node, ProblemBuilder, Vehicle};
use crate::solver::genetic::GeneticAlgorithm;
use crate::solver::{Context, Metaheuristic, SearchHistory};

pub struct WasmProblem {
    pub nodes: Vec<Node>,
    pub vehicle: Option<Vehicle>,
}

impl WasmProblem {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            vehicle: None,
        }
    }
}

#[wasm_bindgen]
pub struct Solver {
    ctx: Option<Context>,
    config: Config,
    wasm_problem: WasmProblem,
    metaheuristic: Option<GeneticAlgorithm>,
    best_cost: f64,
}

#[wasm_bindgen]
impl Solver {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            ctx: None,
            config: Config::default(),
            wasm_problem: WasmProblem::new(),
            metaheuristic: None,
            best_cost: f64::INFINITY,
        }
    }

    pub fn clear(&mut self) {
        self.wasm_problem = WasmProblem::new();
        self.ctx = None;
        self.metaheuristic = None;
        self.best_cost = f64::INFINITY;
    }

    pub fn add_node(&mut self, id: usize, demand: i32, x: i32, y: i32) {
        let new_node = Node {
            id,
            coord: Coordinate { lng: x, lat: y },
            demand,
        };
        self.wasm_problem.nodes.push(new_node);
    }

    pub fn add_capacity(&mut self, capacity: i32) {
        self.wasm_problem.vehicle = Some(Vehicle {
            id: 0,
            cap: capacity,
        });
    }

    pub fn load_problem(&mut self) {
        let start_time = Instant::now();
        // Initialize search history
        let search_history = SearchHistory::new(start_time);

        let problem_builder = ProblemBuilder::new(
            self.wasm_problem.nodes.clone(),
            self.wasm_problem.vehicle.take().expect("No vehicle"),
        );
        let mut config = self.config.clone();

        let problem = problem_builder.build(&mut config);

        let ctx = Context::new(problem, config, search_history);
        self.metaheuristic = Some(GeneticAlgorithm::new(&ctx));
        self.ctx = Some(ctx);
    }

    pub fn update_time_limit(&mut self, value: i32) {
        self.config.time_limit = value as u64;
    }

    pub fn update_min_population_size(&mut self, value: i32) {
        self.config.min_population_size = value as u64;
    }

    pub fn update_initial_individuals(&mut self, value: i32) {
        self.config.initial_individuals = value as u64;
    }

    pub fn update_generation_size(&mut self, value: i32) {
        self.config.population_lambda = value as u64;
    }

    pub fn update_local_search_granularity(&mut self, value: i32) {
        self.config.local_search_granularity = value as u64;
    }

    pub fn update_number_of_elites(&mut self, value: i32) {
        self.config.num_elites = value as u64;
    }

    pub fn update_feasibility_proportion_target(&mut self, value: f64) {
        self.config.feasibility_proportion_target = value;
    }

    pub fn update_rr_start_temp(&mut self, value: f64) {
        self.config.rr_start_temp = value;
    }

    pub fn update_rr_gamma(&mut self, value: f64) {
        self.config.rr_gamma = value;
    }

    pub fn update_elite_start_temp(&mut self, value: f64) {
        self.config.elite_education_start_temp = value;
    }

    pub fn update_elite_gamma(&mut self, value: f64) {
        self.config.elite_education_gamma = value;
    }

    pub fn update_elite_education(&mut self, value: bool) {
        self.config.elite_education = value;
    }

    pub fn iterate(&mut self) -> JsValue {
        if let Some(genetic) = self.metaheuristic.as_mut() {
            if !genetic.terminated() {
                genetic.iterate(&self.ctx.as_ref().unwrap());
            }
        }

        if let Some(entry) = self
            .ctx
            .as_ref()
            .unwrap()
            .search_history
            .borrow()
            .last_entry()
        {
            if entry.solution.cost + EPSILON < self.best_cost {
                self.best_cost = entry.solution.cost;
                return JsValue::from_serde(&entry.solution).unwrap();
            }
        }
        return JsValue::NULL;
    }
}
