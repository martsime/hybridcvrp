use std::collections::HashSet;
use std::time::Instant;

use crate::constants::EPSILON;
use crate::models::FloatType;
use crate::solver::genetic::{Individual, Population, Split};
use crate::solver::improvement::{LocalSearch, RuinRecreate};
use crate::solver::{Context, Metaheuristic, SearchHistory};

pub struct GeneticAlgorithm {
    pub population: Population,
    pub ls: LocalSearch,
    pub rr: RuinRecreate,
    pub split: Split,
    pub iterations: u64,
    pub next_penalty_update: u64,
    pub next_log_interval: u64,

    pub best_solution: Option<Individual>,
    pub best_iteration: u64,
    pub diversified_start: u64,
    pub diversify: bool,
    pub current_best_solution_cost: FloatType,
    pub search_history: SearchHistory,
    pub diversity: FloatType,
}

impl GeneticAlgorithm {
    pub fn new(ctx: &Context, search_history: SearchHistory) -> Self {
        Self {
            population: Population::new(ctx),
            split: Split::new(ctx),
            ls: LocalSearch::new(ctx, 1.0),
            rr: RuinRecreate::new(ctx),
            iterations: 0,
            // Update penalty at this iteration
            next_penalty_update: 0,
            next_log_interval: 0,
            diversified_start: 0,
            diversify: false,

            best_solution: None,
            current_best_solution_cost: FloatType::INFINITY,
            best_iteration: 0,
            search_history,
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
            .count() as FloatType
            / self.population.feasible_history.len() as FloatType;

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
                0.0001f64.max(10_000_000.0f64.min(config.penalty_capacity as f64)) as FloatType;
        }

        for individual in self.population.infeasible.population.iter_mut() {
            individual.evaluate(ctx);
        }

        self.population.infeasible.population.sort();
    }

    fn educate(&mut self, ctx: &Context, child: &mut Individual) {
        // Local search
        if ctx.config.borrow().ls_enabled {
            // let start_time = Instant::now();
            self.ls.run(ctx, child, 1.0);
            // ctx.meta.add_duration("LS", start_time.elapsed());
        }

        let rnd = ctx.random.real();
        if ctx.config.borrow().rr_mutation && rnd < ctx.config.borrow().rr_probability {
            let cost_before = child.penalized_cost();
            // let start_time = Instant::now();
            self.rr
                .run(ctx, child, &mut self.search_history, cost_before);
            // ctx.meta.add_duration("R&R", start_time.elapsed());
            if self.rr.best_cost() + EPSILON < self.current_best_solution_cost
                || ctx.random.real() < (1.0 - ctx.config.borrow().rr_acceptance_alpha)
            {
                self.rr.get_best_solution(child);
            } else {
                self.rr.get_solution(child);
            }
        }

        // Repair with probability
        if !child.is_feasible() && ctx.random.real() < ctx.config.borrow().repair_probability {
            let mut repaired_child = child.clone();
            // repaired_child.number += self.population.total_individuals_count;
            if ctx.config.borrow().ls_enabled {
                // let start_time = Instant::now();
                self.ls.run(ctx, &mut repaired_child, 10.0);
                // ctx.meta.add_duration("LS Repair", start_time.elapsed());
            }
            if repaired_child.is_feasible() {
                self.update_best(ctx, &repaired_child);
                self.population.add_individual(ctx, repaired_child, false);
            }
        }

        // Update best solution
        self.update_best(ctx, &child);
    }

    fn log(&mut self, ctx: &Context) {
        self.next_log_interval += ctx.config.borrow().log_interval;
        let mut log_text = String::new();
        log_text.push_str(&format!(
            "T(s): {:.2} | ",
            self.search_history.start_time.elapsed().as_secs_f64()
        ));
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
            self.population.feasible.get_diversity(ctx) / (ctx.problem.dim() - 1) as FloatType;
        log_text.push_str(&format!(
            "Div {:.2} {:.2} | ",
            self.diversity,
            self.population.infeasible.get_diversity(ctx) / (ctx.problem.dim() - 1) as FloatType
        ));
        log_text.push_str(&format!(
            "Feas {:.2} | ",
            self.population.history_fraction()
        ));
        log_text.push_str(&format!("Pen {:.2}", ctx.config.borrow().penalty_capacity));
        log::info!("{}", log_text);
    }

    fn update_best(&mut self, ctx: &Context, individual: &Individual) {
        if individual.is_feasible() && individual.penalized_cost() < self.current_best_solution_cost
        {
            self.best_iteration = self.iterations;
            self.current_best_solution_cost = individual.penalized_cost();
            if self.current_best_solution_cost < self.search_history.best_cost {
                self.best_solution = Some(individual.clone());
                self.search_history
                    .add_message(format!("New best: {:.2}", individual.penalized_cost()));
                self.search_history.add(ctx, &individual);
            }
        }
    }

    fn reset(&mut self, ctx: &Context) {
        self.search_history.add_message(format!("Resetting"));
        self.population = Population::new(ctx);
        self.next_penalty_update = self.iterations;
        self.next_log_interval = self.iterations;
        self.current_best_solution_cost = FloatType::INFINITY;
        self.best_iteration = self.iterations;
        self.init(ctx);
    }
}

impl Metaheuristic for GeneticAlgorithm {
    fn iterate(&mut self, ctx: &Context) -> bool {
        // Select two parents and perform crossover
        // let start_time = Instant::now();
        let parent_one = self.population.get_parent(ctx);
        let parent_two = self.population.get_parent(ctx);
        let mut child = self.crossover(ctx, parent_one, parent_two);
        // ctx.meta.add_duration("Recombination", start_time.elapsed());

        // Max number of routes the child is allowed to get
        let max_routes = parent_one.num_nonempty_routes();
        self.split.run(ctx, &mut child, max_routes as u64);

        // Educate child
        // let start_time = Instant::now();
        self.educate(ctx, &mut child);
        // ctx.meta.add_duration("Education", start_time.elapsed());

        // let start_time = Instant::now();

        // Add child to population
        self.population.add_individual(ctx, child, true);

        // Update penalties at interval
        if self.iterations >= self.next_penalty_update {
            self.update_penalty(ctx);
        }

        // Log at interval
        if self.iterations >= self.next_log_interval {
            self.log(ctx);
        }

        if self.iterations - self.best_iteration
            > ctx.config.borrow().max_iterations_without_improvement
        {
            self.reset(ctx);
        }

        let config = ctx.config.borrow();

        if let Some(max_iter) = config.max_iterations {
            if max_iter <= self.iterations {
                return true;
            }
        }

        // Update number of iterations
        self.iterations += 1;
        // Return true if termination
        self.search_history.start_time.elapsed().as_secs() >= ctx.config.borrow().time_limit
    }

    fn init(&mut self, ctx: &Context) {
        let num = ctx.config.borrow().initial_individuals;
        let max_routes = ctx.config.borrow().num_vehicles as usize;
        if ctx.config.borrow().elite_education
            && ctx.problem.num_customers() > ctx.config.borrow().dive_problem_size_limit
        {
            self.search_history.add_message(format!("Elite Education"));
            let mut child = Individual::new_random(ctx, 0);
            let mut penalty_multiplier = 1.0;
            self.split.run(ctx, &mut child, max_routes as u64);
            self.educate(ctx, &mut child);

            while !child.is_feasible() {
                if penalty_multiplier < 1000.0 {
                    penalty_multiplier *= 5.0;
                    self.ls.run(ctx, &mut child, penalty_multiplier);
                } else {
                    child = Individual::new_random(ctx, 0);
                    penalty_multiplier = 1.0;
                    self.split.run(ctx, &mut child, max_routes as u64);
                    self.educate(ctx, &mut child);
                }
            }
            let cost_limit = child.penalized_cost();
            self.rr.setup_dive(ctx);
            self.rr
                .run(ctx, &mut child, &mut self.search_history, cost_limit);
            self.rr.get_best_solution(&mut child);
            self.update_best(ctx, &child);
            self.population.add_individual(ctx, child, false);
            self.search_history
                .add_message(format!("Elite Education Complete"));
            self.rr.setup_mutation(ctx);
            self.log(ctx);
        }

        self.search_history
            .add_message(format!("Generating population"));
        for _ in 0..num {
            // Create random individual
            let indiviual_number = self.population.total_individuals_count;
            let mut child = Individual::new_random(ctx, indiviual_number);
            self.split.run(ctx, &mut child, max_routes as u64);
            self.educate(ctx, &mut child);
            self.population.add_individual(ctx, child, true);
        }
        self.search_history
            .add_message(format!("Population generated"));
    }

    fn history(&self) -> &SearchHistory {
        &self.search_history
    }

    fn print(&self) {
        log::info!(
            "Cost of best solution found: {:.2}",
            self.search_history.best_cost
        );
    }
}
