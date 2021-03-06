mod node;
pub use self::node::*;

mod recreate;
pub use self::recreate::*;

mod ruin;
pub use self::ruin::*;

mod schedule;
pub use self::schedule::*;

use std::collections::HashSet;

use ahash::RandomState;
use instant::Duration;

use crate::solver::genetic::Individual;
use crate::solver::Context;
use crate::utils::FloatCompare;

#[derive(Debug, Clone)]
pub struct RuinRecreateSolution {
    pub routes: Vec<Route>,
    pub unassigned: Vec<usize>,
    pub ruined_routes: HashSet<usize, RandomState>,
    pub locations: Vec<NodeLocation>,
    pub cost: f64,
}

impl RuinRecreateSolution {
    pub fn new(ctx: &Context) -> Self {
        let dim = ctx.problem.dim();
        let max_num_routes = ctx.config.borrow().num_vehicles as usize;

        let routes = vec![Route::empty(); max_num_routes];
        let unassigned: Vec<usize> = Vec::with_capacity(dim);
        let locations = vec![NodeLocation::empty(); dim];

        Self {
            routes,
            unassigned,
            ruined_routes: HashSet::with_hasher(RandomState::new()),
            locations,
            cost: f64::INFINITY,
        }
    }

    pub fn from(&mut self, other: &Self) {
        self.cost = other.cost;
        for (index, loc) in self.locations.iter_mut().enumerate() {
            loc.update_from_other(&other.locations[index]);
        }
        for (route_number, route) in self.routes.iter_mut().enumerate() {
            route.update_from_other(&other.routes[route_number]);
        }
    }

    pub fn load(&mut self, individual: &Individual) {
        assert_eq!(
            individual.phenotype.len(),
            self.routes.len(),
            "Number of routes are not equal"
        );
        let evaluation = &individual.evaluation;
        for (route_number, route) in self.routes.iter_mut().enumerate() {
            // Update the route
            route.nodes.clear();
            route
                .nodes
                .extend(individual.phenotype[route_number].iter());
            route.distance = evaluation.routes[route_number].distance;
            route.overload = evaluation.routes[route_number].overload;

            // Update the location for the nodes on the route
            for (node_index, node) in route.nodes.iter().enumerate() {
                self.locations[*node].update(route_number, node_index);
            }
        }
    }

    pub fn is_feasible(&self) -> bool {
        let total_overload: f64 = self
            .routes
            .iter()
            .map(|route| 0f64.max(route.overload))
            .sum();
        total_overload.approx_eq(0.0)
    }

    pub fn evaluate<'a, I>(&mut self, ctx: &Context, updated_routes: I)
    where
        I: Iterator<Item = &'a usize>,
    {
        let penalty_capacity = ctx.config.borrow().penalty_capacity;
        self.cost = self
            .routes
            .iter()
            .map(|route| route.distance + 0f64.max(route.overload) * penalty_capacity)
            .sum();

        for &route_index in updated_routes {
            for node_index in 0..self.routes[route_index].nodes.len() {
                let node = self.routes[route_index].nodes[node_index];
                self.locations[node].update(route_index, node_index);
            }
        }
    }

    pub fn reevaluate(&mut self, ctx: &Context) {
        let penalty_capacity = ctx.config.borrow().penalty_capacity;
        self.cost = self
            .routes
            .iter()
            .map(|route| route.distance + 0f64.max(route.overload) * penalty_capacity)
            .sum();
    }
}

pub struct RuinRecreate {
    pub ctx: &'static Context,
    pub ruin: Box<dyn Ruin>,
    pub recreate: Box<dyn Recreate>,
    pub solution: RuinRecreateSolution,
    pub current_solution: RuinRecreateSolution,
    pub best_solution: Option<RuinRecreateSolution>,
    pub acceptance: AcceptanceCriterion,
    pub update_penalty: bool,
}

impl RuinRecreate {
    pub fn new(ctx: &Context) -> Self {
        let mut rr = Self {
            ctx: unsafe { &*(ctx as *const Context) },
            ruin: Box::new(AdjacentStringRemoval::new(ctx)),
            recreate: Box::new(GreedyBlink::default()),
            solution: RuinRecreateSolution::new(ctx),
            current_solution: RuinRecreateSolution::new(ctx),
            best_solution: None,
            acceptance: IterationSchedule::new(100.0, 100).into(),
            update_penalty: false,
        };
        rr.setup_mutation(ctx);
        rr
    }

    pub fn setup_elite_education(&mut self, ctx: &Context) {
        let config = ctx.config.borrow();
        if config.elite_education_time_based {
            let duration_f64 = config.time_limit as f64 * config.elite_education_time_fraction;
            let duration = Duration::from_secs_f64(duration_f64);
            self.acceptance = TimeSchedule::new(config.elite_education_start_temp, duration).into();
        } else {
            let iterations = (config.elite_education_gamma * ctx.problem.num_customers() as f64)
                .round() as usize;
            self.acceptance =
                IterationSchedule::new(config.elite_education_start_temp, iterations).into();
        }
        self.update_penalty = true;
    }

    pub fn set_acceptance(&mut self, acceptance: AcceptanceCriterion) {
        self.acceptance = acceptance;
    }

    pub fn setup_mutation(&mut self, ctx: &Context) {
        let config = ctx.config.borrow();
        let iterations = (config.rr_gamma * ctx.problem.num_customers() as f64).round() as usize;
        self.acceptance = IterationSchedule::new(config.rr_start_temp, iterations).into();
        self.update_penalty = false;
    }

    pub fn load(&mut self, ctx: &Context, individual: &mut Individual) {
        // Update data
        self.ctx = unsafe { &*(ctx as *const Context) };

        // Load solution
        self.solution.load(individual);
        self.current_solution = self.solution.clone();
        let routes: Vec<usize> = (0..self.solution.routes.len()).into_iter().collect();
        self.solution.evaluate(self.ctx, routes.iter());
        self.acceptance.reset();
        // self.acceptance.print();

        self.best_solution = Some(self.solution.clone());
    }

    pub fn complete(&self) -> bool {
        self.acceptance.completed()
    }

    fn update_best(&mut self) {
        self.best_solution = Some(self.current_solution.clone());
        let mut search_history = self.ctx.search_history.borrow_mut();
        if self.current_solution.is_feasible()
            && self
                .current_solution
                .cost
                .approx_lt(search_history.best_cost)
        {
            let mut best_individual = Individual::new_random(self.ctx, 0);
            self.update_individual(&self.current_solution, &mut best_individual);
            search_history
                .add_message(format!("New best: {:.2}", best_individual.penalized_cost()));
            search_history.add(self.ctx, &best_individual);
        }
    }

    pub fn search(&mut self) {
        let update_interval = 1000;
        for i in 1..=update_interval {
            // Check for possible update of penalty
            if i == update_interval {
                // Update penalty if enabled in search
                if self.update_penalty {
                    {
                        let mut config = self.ctx.config.borrow_mut();
                        if self.solution.is_feasible() {
                            config.penalty_capacity *= config.penalty_dec_multiplier;
                        } else {
                            config.penalty_capacity *= config.penalty_inc_multiplier;
                        }
                    }

                    // Reevaluate the solutions
                    if let Some(best_solution) = self.best_solution.as_mut() {
                        best_solution.reevaluate(self.ctx);
                    }
                    self.solution.reevaluate(self.ctx);
                    self.current_solution.reevaluate(self.ctx);
                }
            }

            // Perform R&R move
            let cost_before = self.current_solution.cost;
            self.ruin.run(self.ctx, &mut self.current_solution);
            self.recreate.run(self.ctx, &mut self.current_solution);
            if self
                .acceptance
                .accept(self.current_solution.cost, cost_before, &self.ctx.random)
            {
                if let Some(best_solution) = self.best_solution.as_ref() {
                    if self.current_solution.cost.approx_lt(best_solution.cost) {
                        self.update_best();
                    }
                } else {
                    self.update_best();
                }
                self.solution.from(&self.current_solution);
            }
            self.current_solution.from(&self.solution);
            self.acceptance.update();
            if self.acceptance.completed() {
                break;
            }
        }
    }

    pub fn get_solution(&self, individual: &mut Individual) {
        self.update_individual(&self.solution, individual);
    }

    pub fn get_best_solution(&self, individual: &mut Individual) {
        if let Some(best_solution) = self.best_solution.as_ref() {
            self.update_individual(&best_solution, individual);
        }
    }

    pub fn best_cost(&self) -> f64 {
        match self.best_solution.as_ref() {
            Some(best_solution) => best_solution.cost,
            None => f64::INFINITY,
        }
    }

    fn update_individual(&self, solution: &RuinRecreateSolution, individual: &mut Individual) {
        assert_eq!(
            0,
            solution.unassigned.len(),
            "Can only update individual if no customers are unassigned"
        );
        individual.genotype.clear();
        for (route_number, route) in solution.routes.iter().enumerate() {
            individual.phenotype[route_number] = route.nodes.clone();
            individual.genotype.extend(route.nodes.clone());
        }

        // Reevaluate the individual
        individual.sort_routes(self.ctx);
        individual.evaluate(self.ctx);
    }
}
