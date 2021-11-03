use std::collections::HashSet;

use crate::solver::genetic::{Individual, Population, Split};
use crate::solver::improvement::{LocalSearch, RuinRecreate};
use crate::solver::{Context, Metaheuristic};
use crate::utils::FloatCompare;

#[derive(PartialEq)]
pub enum State {
    Created,
    EliteEducation,
    Initialization,
    Cycle,
    Terminated,
}

pub struct GeneticAlgorithm {
    pub state: State,
    pub population: Population,
    pub ls: LocalSearch,
    pub rr: RuinRecreate,
    pub split: Split,
    pub iterations: u64,
    pub next_penalty_update: u64,
    pub next_log_interval: u64,

    pub child: Individual,
    pub num_initialized: u64,

    pub best_solution: Option<Individual>,
    pub best_iteration: u64,
    pub diversified_start: u64,
    pub diversify: bool,
    pub current_best_solution_cost: f64,
    pub diversity: f64,
}

impl GeneticAlgorithm {
    pub fn new(ctx: &Context) -> Self {
        Self {
            state: State::Created,
            population: Population::new(ctx),
            split: Split::new(ctx),
            ls: LocalSearch::new(ctx, 1.0),
            rr: RuinRecreate::new(ctx),
            iterations: 0,
            child: Individual::new_random(ctx, 0),
            num_initialized: 0,
            // Update penalty at this iteration
            next_penalty_update: 0,
            next_log_interval: 0,
            diversified_start: 0,
            diversify: false,

            best_solution: None,
            current_best_solution_cost: f64::INFINITY,
            best_iteration: 0,
            diversity: 1.0,
        }
    }

    fn crossover(
        &self,
        ctx: &Context,
        parent_one: &Individual,
        parent_two: &Individual,
    ) -> Individual {
        // Randomly select start of the crossover zone
        let start = ctx.random.range_usize(0, parent_one.genotype.len());

        // Randomly select end of the crossover zone different from the start
        let mut end;
        loop {
            end = ctx.random.range_usize(0, parent_one.genotype.len());
            if start != end {
                break;
            }
        }

        // Use the OX crossover
        self.crossover_ox(ctx, parent_one, parent_two, start, end)
    }

    fn crossover_ox(
        &self,
        ctx: &Context,
        parent_one: &Individual,
        parent_two: &Individual,
        start: usize,
        end: usize,
    ) -> Individual {
        // Helper function to wrap indices in the vector around
        #[inline]
        fn wrap_index(index: usize, length: usize) -> usize {
            if index == length {
                0
            } else {
                index
            }
        }
        // Clone the genotype of the first parent
        let mut genotype = parent_one.genotype.clone();

        // Set with all the genes added to the genotype
        let mut added_genes =
            HashSet::with_capacity_and_hasher(genotype.len(), ctx.random.random_state());

        // Index into the genotype
        let mut index = start;

        loop {
            // Add the gene from parent one into the set of genes
            added_genes.insert(genotype[index]);

            // Done with parent one
            if index == end {
                index = wrap_index(index + 1, genotype.len());
                break;
            }

            // Move index one to the right, possibly wrapping around
            index = wrap_index(index + 1, genotype.len());
        }

        // Add genes from parent_two until all genes have been added
        for &parent_two_gene in parent_two.genotype.iter() {
            // Check to see if the gene is already added
            if !added_genes.contains(&parent_two_gene) {
                // Add the parent two gene to the set of added genes
                added_genes.insert(parent_two_gene);
                // Update the childs genotype with the gene from parent two
                genotype[index] = parent_two_gene;
                // Update the index into the childs genotype
                index = wrap_index(index + 1, genotype.len());
            }
        }
        // Return the child
        Individual::new(genotype, self.population.total_individuals_count)
    }

    fn update_penalty(&mut self, ctx: &Context) {
        // Set the next iteration in which the penalty should be updated
        self.next_penalty_update += ctx.config.borrow().penalty_update_interval;
        // Calculate the fraction of the population that was feasible since last time
        let feasible_fraction = self
            .population
            .feasible_history
            .iter()
            .filter(|&&b| b)
            .count() as f64
            / self.population.feasible_history.len() as f64;

        {
            // Update the penalty
            let mut config = ctx.config.borrow_mut();
            if feasible_fraction < config.feasibility_proportion_target - 0.05 {
                config.penalty_capacity *= config.penalty_inc_multiplier;
            } else if feasible_fraction > config.feasibility_proportion_target + 0.05 {
                config.penalty_capacity *= config.penalty_dec_multiplier;
            }

            // Make sure the penalty is in the range [0.01, 10_000.0]
            config.penalty_capacity =
                0.0001f64.max(10_000_000.0f64.min(config.penalty_capacity as f64));
        }

        for individual in self.population.infeasible.population.iter_mut() {
            individual.evaluate(ctx);
        }

        self.population.infeasible.population.sort();
    }

    fn educate(&mut self, ctx: &Context) {
        // Local search
        let child = &mut self.child;
        if ctx.config.borrow().ls_enabled {
            self.ls.run(ctx, child, 1.0);
        }

        // R&R search
        let rnd = ctx.random.real();
        if ctx.config.borrow().rr_mutation && rnd < ctx.config.borrow().rr_probability {
            self.rr.load(ctx, child);
            while !self.rr.complete() {
                self.rr.search();
            }
            self.rr.get_best_solution(child);
        }

        // Repair with probability using local search with higher penalty
        if !child.is_feasible() && ctx.random.real() < ctx.config.borrow().repair_probability {
            let unrepaired_child = child.clone();
            if ctx.config.borrow().ls_enabled {
                self.ls.run(ctx, &mut self.child, 10.0);
            }
            if self.child.is_feasible() {
                self.update_best(ctx);
                self.population
                    .add_individual(ctx, self.child.clone(), false);
            }
            self.child = unrepaired_child;
        }

        // Update best solution
        self.update_best(ctx);
    }

    fn log(&mut self, ctx: &Context) {
        self.next_log_interval += ctx.config.borrow().log_interval;
        let mut log_text = String::new();
        log_text.push_str(&format!("T(s): {:.2} | ", ctx.elapsed_as_secs_f64()));
        log_text.push_str(&format!(
            "Iter: {:6} {:4} | ",
            self.iterations,
            self.iterations - self.best_iteration
        ));
        log_text.push_str(&format!(
            "Feas {} {:.2} {:.2} | ",
            self.population.feasible.size(),
            self.population.feasible.get_best_cost(),
            self.population.feasible.get_average_cost(ctx)
        ));
        log_text.push_str(&format!(
            "Inf {} {:.2} {:.2} | ",
            self.population.infeasible.size(),
            self.population.infeasible.get_best_cost(),
            self.population.infeasible.get_average_cost(ctx)
        ));
        self.diversity =
            self.population.feasible.get_diversity(ctx) / (ctx.problem.dim() - 1) as f64;
        log_text.push_str(&format!(
            "Div {:.2} {:.2} | ",
            self.diversity,
            self.population.infeasible.get_diversity(ctx) / (ctx.problem.dim() - 1) as f64
        ));
        log_text.push_str(&format!(
            "Feas {:.2} | ",
            self.population.history_fraction()
        ));
        log_text.push_str(&format!("Pen {:.2}", ctx.config.borrow().penalty_capacity));
        log::debug!("{}", log_text);
    }

    fn update_best(&mut self, ctx: &Context) {
        if self.child.is_feasible()
            && self
                .child
                .penalized_cost()
                .approx_lt(self.current_best_solution_cost)
        {
            self.best_iteration = self.iterations;
            self.current_best_solution_cost = self.child.penalized_cost();
            let mut search_history = ctx.search_history.borrow_mut();
            if self
                .current_best_solution_cost
                .approx_lt(search_history.best_cost)
            {
                self.best_solution = Some(self.child.clone());
                search_history.add_message(format!("New best: {:.2}", self.child.penalized_cost()));
                search_history.add(ctx, &self.child);
            }
        }
    }

    fn reset(&mut self, ctx: &Context) {
        ctx.search_history
            .borrow_mut()
            .add_message(format!("Resetting"));
        self.population = Population::new(ctx);
        self.next_penalty_update = self.iterations;
        self.next_log_interval = self.iterations;
        self.current_best_solution_cost = f64::INFINITY;
        self.best_iteration = self.iterations;
        self.state = State::Created;
    }
}

impl Metaheuristic for GeneticAlgorithm {
    fn iterate(&mut self, ctx: &Context) {
        if ctx.terminate() {
            self.state = State::Terminated;
        }
        match self.state {
            State::Created => {
                if ctx.config.borrow().elite_education
                    && ctx.problem.num_customers()
                        > ctx.config.borrow().elite_education_problem_size_limit
                {
                    // Setup Elite Education
                    self.state = State::EliteEducation;
                    let max_routes = ctx.config.borrow().num_vehicles as usize;
                    self.split.run(ctx, &mut self.child, max_routes as u64);
                    self.educate(ctx);
                    self.rr.setup_elite_education(ctx);
                    self.rr.load(ctx, &mut self.child);
                } else {
                    self.state = State::Initialization;
                }
            }
            State::EliteEducation => {
                if !self.rr.complete() {
                    self.rr.search();
                } else {
                    self.state = State::Initialization;
                    self.rr.get_best_solution(&mut self.child);
                    self.update_best(ctx);
                    self.population
                        .add_individual(ctx, self.child.clone(), false);
                    self.rr.setup_mutation(ctx);
                    ctx.reset_penalty();
                    self.log(ctx);
                }
            }
            State::Initialization => {
                if self.num_initialized < ctx.config.borrow().initial_individuals {
                    // Create random individual
                    let max_routes = ctx.config.borrow().num_vehicles;
                    self.child = Individual::new_random(ctx, self.num_initialized);
                    self.split.run(ctx, &mut self.child, max_routes);
                    self.educate(ctx);
                    self.population
                        .add_individual(ctx, self.child.clone(), true);
                    self.num_initialized += 1;
                } else {
                    self.state = State::Cycle;
                }
            }
            State::Cycle => {
                // Select two parents and perform crossover
                let parent_one = self.population.get_parent(ctx);
                let parent_two = self.population.get_parent(ctx);
                self.child = self.crossover(ctx, parent_one, parent_two);

                // Max number of routes the child is allowed to get
                let max_routes = parent_one.num_nonempty_routes();
                self.split.run(ctx, &mut self.child, max_routes as u64);

                // Educate child
                self.educate(ctx);

                // Add child to population
                self.population
                    .add_individual(ctx, self.child.clone(), true);

                // Update penalties at interval
                if self.iterations >= self.next_penalty_update {
                    self.update_penalty(ctx);
                }

                // Log at interval
                if self.iterations >= self.next_log_interval {
                    self.log(ctx);
                }

                // Possible reset of population
                if self.iterations - self.best_iteration > ctx.config.borrow().max_iterations {
                    self.reset(ctx);
                }

                // Update number of iterations
                self.iterations += 1;
            }
            State::Terminated => {}
        }
    }
    fn terminated(&self) -> bool {
        self.state == State::Terminated
    }
}
