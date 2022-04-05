use std::collections::HashSet;

use ahash::RandomState;

use crate::{
    solver::{
        genetic::{GeneticAlgorithm, Individual},
        improvement::{Acceptance, IterationSchedule},
        Context, Metaheuristic,
    },
    utils::FloatCompare,
};

#[derive(PartialEq)]
enum State {
    NotInitialized,
    Cycle,
    Terminated,
}

pub struct DecomposedGeneticAlgorithm {
    state: State,
    global_solution: Individual,
    global_ga: GeneticAlgorithm,
}

impl DecomposedGeneticAlgorithm {
    pub fn new(ctx: &Context) -> Self {
        Self {
            state: State::NotInitialized,
            global_solution: Individual::new_random(ctx, 0),
            global_ga: GeneticAlgorithm::new(ctx),
        }
    }

    fn create_global_solution(&mut self, ctx: &Context) -> Individual {
        log::info!("Creating global solution");
        let max_routes = ctx.config.borrow().num_vehicles;
        let mut child = Individual::new_random(ctx, 0);
        ctx.config.borrow_mut().penalty_capacity *= 1_000_000.0;
        log::info!("Running split");
        self.global_ga.split.run(ctx, &mut child, max_routes);
        self.global_ga.child = child;
        self.global_ga.educate(ctx);
        self.global_ga
            .rr
            .set_acceptance(IterationSchedule::new(10.0, 100_000).into());
        log::info!("Searching in global");
        let rr = &mut self.global_ga.rr;
        rr.load(ctx, &mut self.global_ga.child);
        loop {
            let cost_before = rr.best_cost();
            rr.acceptance.reset();
            while !rr.complete() {
                rr.search();
            }
            rr.get_best_solution(&mut self.global_ga.child);
            if rr.best_cost().approx_gte(cost_before) {
                break;
            }
        }
        child = self.global_ga.child.clone();
        ctx.config.borrow_mut().penalty_capacity /= 1_000_000.0;
        child.evaluate(ctx);
        child
    }
}

impl Metaheuristic for DecomposedGeneticAlgorithm {
    fn iterate(&mut self, ctx: &Context) {
        if ctx.terminate() {
            self.state = State::Terminated;
        }

        match self.state {
            State::NotInitialized => {
                self.global_solution = self.create_global_solution(ctx);
                self.state = State::Cycle;
            }
            State::Cycle => {
                // panic!("{:?}", self.global_solution);
                // println!("{:?}", self.global_solution);
                // self.global_solution.print();
                println!(
                    "Global before decomp: {}",
                    self.global_solution.penalized_cost()
                );
                let mut decomposition = Decomposition::new(&mut self.global_solution, ctx);
                println!(
                    "Sub solution before: {}",
                    decomposition.solution.penalized_cost()
                );
                decomposition.run();
                decomposition.finish(ctx);
                println!(
                    "Global after before: {}",
                    self.global_solution.penalized_cost()
                );
                self.global_ga.child = self.global_solution.clone();
                self.global_ga.update_best(ctx);

                let rr = &mut self.global_ga.rr;
                rr.acceptance.reset();
                rr.load(ctx, &mut self.global_ga.child);
                while !rr.complete() {
                    rr.search();
                }
                rr.get_best_solution(&mut self.global_ga.child);
                self.global_ga.update_best(ctx);
                self.global_solution = self.global_ga.child.clone();
                // decomposition.global.print();
                // println!("{:?}", decomposition.global);
                // decomposition.solution.print();
                // println!("{:?}", decomposition.solution);
                // println!("{:?}", decomposition.mapping);
            }
            State::Terminated => {
                return;
            }
        }
    }

    fn terminated(&self) -> bool {
        self.state == State::Terminated
    }
}

#[derive(Debug)]
pub struct Decomposition<'a> {
    pub global: &'a mut Individual,
    pub mapping: Vec<usize>,
    pub ctx: Context,
    pub solution: Individual,
}

impl<'a> Decomposition<'a> {
    pub fn new(global: &'a mut Individual, ctx: &Context) -> Self {
        let (mut sub_solution, mapping) = Self::decompose_global(&mut *global, ctx);
        let sub_ctx = ctx.from_mapping(&mapping[..]);
        sub_solution.sort_routes(&sub_ctx);
        sub_solution.evaluate(&sub_ctx);
        println!("Decomposed size: {}", sub_ctx.problem.dim());
        Self {
            global,
            mapping,
            ctx: sub_ctx,
            solution: sub_solution,
        }
    }

    pub fn run(&mut self) {
        {
            let mut config = self.ctx.config.borrow_mut();
            config.elite_education = false;
            config.max_iterations = Some(2000);
            println!("{:?}", config);
        }

        let mut sub_ga = GeneticAlgorithm::new(&self.ctx);
        sub_ga.add_initial(&self.ctx, self.solution.clone());

        while !sub_ga.terminated() {
            sub_ga.iterate(&self.ctx);
        }

        self.solution = sub_ga.best_solution.unwrap();
    }

    pub fn finish(self, ctx: &Context) {
        for route in self.solution.phenotype.iter() {
            let new_route: Vec<usize> = route
                .iter()
                .map(|&customer| self.mapping[customer])
                .collect();
            if !new_route.is_empty() {
                self.global.phenotype.push(new_route);
            }
        }
        self.global.genotype = self.global.phenotype.iter().flatten().copied().collect();
        self.global.sort_routes(ctx);
        self.global.evaluate(ctx);
    }

    fn decompose_global(global: &mut Individual, ctx: &Context) -> (Individual, Vec<usize>) {
        let mut route_for_customer = vec![0; ctx.problem.dim()];
        for (route_number, route) in global.phenotype.iter().enumerate() {
            for &customer in route.iter() {
                route_for_customer[customer] = route_number;
            }
        }
        let seed_customer = ctx.random.range_usize(0, global.genotype.len());

        let mut selected_customers: HashSet<usize, RandomState> =
            HashSet::with_capacity_and_hasher(200, ctx.random.random_state());
        let mut selected_routes: HashSet<usize, RandomState> =
            HashSet::with_capacity_and_hasher(50, ctx.random.random_state());

        let decompose_size = ctx.config.borrow().decomposed_problem_min_size as usize;

        let neighbors: Vec<usize> = vec![seed_customer]
            .into_iter()
            .chain(
                ctx.matrix_provider
                    .correlation
                    .get(seed_customer)
                    .iter()
                    .cloned(),
            )
            .collect();
        // println!("Neighbors: {:?}", neighbors);

        for neighbor in neighbors {
            if selected_customers.len() >= decompose_size {
                break;
            }
            let route_index = route_for_customer[neighbor];
            let route = &global.phenotype[route_index];
            if selected_routes.contains(&route_index) {
                continue;
            }

            selected_customers.extend(route.iter());
            selected_routes.insert(route_index);
        }

        let mut mapping = vec![0; selected_customers.len() + 1];

        let mut new_phenotype: Vec<Vec<usize>> = selected_routes
            .iter()
            .map(|&route_index| global.phenotype[route_index].clone())
            .collect();

        let mut index = 1;
        for route in new_phenotype.iter_mut() {
            for customer in route.iter_mut() {
                mapping[index] = *customer;
                *customer = index;
                index += 1;
            }
        }

        let new_genotype: Vec<usize> = new_phenotype.iter().flatten().copied().collect();
        let extra_empty_routes = ((new_phenotype.len() as f64 / 10.0) + 2.0).round() as usize;
        for _ in 0..extra_empty_routes {
            new_phenotype.push(Vec::new());
        }

        let mut sub_solution = Individual::empty();
        sub_solution.genotype = new_genotype;
        sub_solution.phenotype = new_phenotype;

        global.genotype = global
            .genotype
            .iter()
            .filter(|&customer| !selected_customers.contains(&customer))
            .copied()
            .collect();

        global.phenotype = global
            .phenotype
            .iter()
            .enumerate()
            .filter_map(|(route_index, route)| {
                if selected_routes.contains(&route_index) {
                    None
                } else {
                    Some(route.clone())
                }
            })
            .collect();
        (sub_solution, mapping)
    }
}
