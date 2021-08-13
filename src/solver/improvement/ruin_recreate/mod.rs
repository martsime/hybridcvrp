mod recreate;
pub use self::recreate::*;

mod ruin;
pub use self::ruin::*;

use std::collections::HashSet;

use ahash::RandomState;

use crate::constants::EPSILON;
use crate::models::{FloatType, IntType};
use crate::solver::genetic::Individual;
use crate::solver::Context;

#[derive(Debug, Clone)]
pub struct NodeLocation {
    // Index into the vec of routes
    pub route_index: usize,
    // Index into the route
    pub node_index: usize,
}

impl NodeLocation {
    pub fn new(route_index: usize, node_index: usize) -> Self {
        Self {
            route_index,
            node_index,
        }
    }
    pub fn empty() -> Self {
        Self {
            route_index: 0,
            node_index: 0,
        }
    }

    pub fn update(&mut self, route_index: usize, node_index: usize) {
        self.route_index = route_index;
        self.node_index = node_index;
    }

    pub fn update_from_other(&mut self, other: &Self) {
        self.route_index = other.route_index;
        self.node_index = other.node_index;
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    // Nodes on the route in order
    pub nodes: Vec<usize>,

    // Distance of the route
    pub distance: IntType,

    // Total overload on the route
    pub overload: IntType,
}

impl Route {
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            distance: 0,
            overload: 0,
        }
    }
    pub fn remove(&mut self, index: usize, ctx: &Context) -> usize {
        let prev_node = if index == 0 { 0 } else { self.nodes[index - 1] };
        let next_node = if index == self.nodes.len() - 1 {
            0
        } else {
            self.nodes[index + 1]
        };

        // Update distance and overload
        self.distance += -ctx.problem.distance.get(prev_node, self.nodes[index])
            - ctx.problem.distance.get(self.nodes[index], next_node)
            + ctx.problem.distance.get(prev_node, next_node);

        self.overload -= ctx.problem.nodes[self.nodes[index]].demand;

        self.nodes.remove(index)
    }

    pub fn delta_distance(&self, index: usize, node: usize, ctx: &Context) -> IntType {
        let prev_node = if index == 0 { 0 } else { self.nodes[index - 1] };
        let next_node = if index == self.nodes.len() {
            0
        } else {
            self.nodes[index]
        };

        // Update distance and overload
        let delta = -ctx.problem.distance.get(prev_node, next_node)
            + ctx.problem.distance.get(prev_node, node)
            + ctx.problem.distance.get(node, next_node);
        delta
    }

    pub fn add(&mut self, index: usize, node: usize, ctx: &Context) {
        // Update distance and overload
        self.distance += self.delta_distance(index, node, ctx);

        self.overload += ctx.problem.nodes[node].demand;

        self.nodes.insert(index, node);
    }

    pub fn update_from_other(&mut self, other: &Self) {
        self.distance = other.distance;
        self.overload = other.overload;
        self.nodes.clear();
        for node in other.nodes.iter() {
            self.nodes.push(*node);
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuinRecreateSolution {
    pub routes: Vec<Route>,
    pub unassigned: Vec<usize>,
    pub ruined_routes: HashSet<usize, RandomState>,
    pub locations: Vec<NodeLocation>,
    pub cost: FloatType,
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
            cost: FloatType::INFINITY,
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

    pub fn print(&self) {
        for (i, route) in self.routes.iter().enumerate() {
            log::debug!("Route {}: {:?}", i, route.nodes);
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
        let total_overload: IntType = self.routes.iter().map(|route| 0.max(route.overload)).sum();
        total_overload == 0
    }

    pub fn evaluate<'a, I>(&mut self, ctx: &Context, updated_routes: I)
    where
        I: Iterator<Item = &'a usize>,
    {
        let penalty_capacity = ctx.config.borrow().penalty_capacity;
        self.cost = self
            .routes
            .iter()
            .map(|route| {
                route.distance as FloatType + 0.max(route.overload) as FloatType * penalty_capacity
            })
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
            .map(|route| {
                route.distance as FloatType + 0.max(route.overload) as FloatType * penalty_capacity
            })
            .sum();
    }
}

pub struct RuinRecreate {
    pub ctx: &'static Context,
    pub ruin: Box<dyn Ruin>,
    pub recreate: Box<dyn Recreate>,
    pub solution: RuinRecreateSolution,
    pub best_solution: Option<RuinRecreateSolution>,
    pub cost_limit: FloatType,
    pub iterations: usize,
    pub min_temp: FloatType,
    pub max_temp: FloatType,
    pub threshold_one: FloatType,
    pub threshold_two: FloatType,
    pub threshold_switch: FloatType,
    pub update_penalty: bool,
}

impl RuinRecreate {
    pub fn new(ctx: &Context) -> Self {
        let mut rr = Self {
            ctx: unsafe { &*(ctx as *const Context) },
            ruin: Box::new(AdjacentStringRemoval::new(ctx)),
            recreate: Box::new(GreedyBlink::default()),
            solution: RuinRecreateSolution::new(ctx),
            best_solution: None,
            cost_limit: FloatType::INFINITY,
            iterations: 0,
            min_temp: 0.0,
            max_temp: 0.0,
            threshold_one: 0.0,
            threshold_two: 0.0,
            threshold_switch: 0.0,
            update_penalty: false,
        };
        rr.setup_mutation(ctx);
        rr
    }

    pub fn setup_dive(&mut self, ctx: &Context) {
        let config = ctx.config.borrow();
        self.iterations = config.elite_education_gamma * ctx.problem.num_customers();
        self.min_temp = config.elite_education_final_temp;
        self.max_temp = config.elite_education_start_temp;
        self.threshold_one = config.dive_threshold_one;
        self.threshold_two = config.dive_threshold_two;
        self.threshold_switch = config.dive_threshold_switch;
        self.update_penalty = true;
    }

    pub fn setup_mutation(&mut self, ctx: &Context) {
        let config = ctx.config.borrow();
        self.iterations =
            (config.rr_gamma * ctx.problem.num_customers() as FloatType).round() as usize;
        self.min_temp = config.rr_final_temp;
        self.max_temp = config.rr_start_temp;
        self.threshold_one = config.rr_threshold_one;
        self.threshold_two = config.rr_threshold_two;
        self.threshold_switch = config.rr_threshold_switch;
        self.update_penalty = false;
    }

    pub fn run(&mut self, ctx: &Context, individual: &mut Individual, cost_limit: FloatType) {
        // Update data
        self.ctx = unsafe { &*(ctx as *const Context) };
        self.cost_limit = cost_limit;

        // Load solution
        self.solution.load(individual);
        let routes: Vec<usize> = (0..self.solution.routes.len()).into_iter().collect();
        self.solution.evaluate(self.ctx, routes.iter());

        self.best_solution = Some(self.solution.clone());

        self.search();
    }

    fn update_best(&mut self, solution: &RuinRecreateSolution) {
        self.cost_limit = solution.cost;
        self.best_solution = Some(solution.clone());
        let mut search_history = self.ctx.search_history.borrow_mut();
        if solution.is_feasible() && solution.cost < search_history.best_cost {
            let mut best_individual = Individual::new_random(self.ctx, 0);
            self.update_individual(solution, &mut best_individual);
            search_history
                .add_message(format!("New best: {:.2}", best_individual.penalized_cost()));
            search_history.add(self.ctx, &best_individual);
        }
    }

    fn search(&mut self) {
        let mut cooling_factor = if self.max_temp < EPSILON {
            1.0
        } else {
            if self.min_temp < EPSILON {
                ((0.000001 / self.max_temp) * 1.0).powf(1.0f64 / self.iterations as f64)
            } else {
                ((self.min_temp / self.max_temp) * 1.0).powf(1.0f64 / self.iterations as f64)
            }
        };
        let mut temp = self.max_temp;
        let threshold_iteration = (self.iterations as f64 * self.threshold_switch).round() as usize;
        let mut solution = self.solution.clone();
        let mut check_run_time = 100;
        let mut update_penalty = 100;
        for i in 0..self.iterations {
            check_run_time -= 1;
            update_penalty -= 1;
            if check_run_time == 0 {
                if self.ctx.terminate() {
                    break;
                }
                check_run_time = 100;
            }
            if update_penalty == 0 {
                if self.update_penalty {
                    // Update penalty
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
                    solution.reevaluate(self.ctx);
                }
                update_penalty = 100;
            }
            let cost_before = solution.cost;
            self.ruin.run(self.ctx, &mut solution);
            self.recreate.run(self.ctx, &mut solution);
            let upper_bound = if i < threshold_iteration {
                self.threshold_one * self.cost_limit
            } else {
                self.threshold_two * self.cost_limit
            };
            if solution.cost + EPSILON < cost_before - temp * self.ctx.random.real().ln()
                && solution.cost + EPSILON < upper_bound
            {
                if solution.cost > cost_before + EPSILON {}
                if let Some(best_solution) = self.best_solution.as_ref() {
                    if solution.cost + EPSILON < best_solution.cost {
                        self.update_best(&solution);
                    }
                } else {
                    self.update_best(&solution);
                }
                self.solution.from(&solution);
            }
            solution.from(&self.solution);
            temp *= cooling_factor;
            if temp < self.min_temp {
                temp = self.min_temp;
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

    pub fn best_cost(&self) -> FloatType {
        match self.best_solution.as_ref() {
            Some(best_solution) => best_solution.cost,
            None => FloatType::INFINITY,
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
