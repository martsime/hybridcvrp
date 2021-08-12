use std::cmp::{min, Ordering};
use std::collections::HashMap;

use ahash::RandomState;

use crate::models::FloatType;
use crate::solver::genetic::Individual;
use crate::solver::Context;

#[derive(Debug)]
pub struct Diversity {
    pub distance: i64,
    pub to_number: u64,
}

impl Diversity {
    pub fn new(distance: i64, to_number: u64) -> Self {
        Self {
            distance,
            to_number,
        }
    }
}

impl PartialOrd for Diversity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl PartialEq for Diversity {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Ord for Diversity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl Eq for Diversity {}

pub struct SubPopulation {
    // Current individuals in the population
    pub population: Vec<Individual>,

    // Matrix with diversity between the individuals
    pub diversity: HashMap<u64, Vec<Diversity>, RandomState>,

    pub focus_diversity: bool,

    // Maximum allowed individuals in the population
    max_individuals: usize,
}

impl SubPopulation {
    pub fn new(ctx: &Context) -> Self {
        let config = ctx.config.borrow();
        let max_individuals = (config.min_population_size + config.population_lambda) as usize;
        Self {
            population: Vec::new(),
            diversity: HashMap::with_capacity_and_hasher(
                max_individuals,
                ctx.random.random_state(),
            ),
            focus_diversity: false,
            max_individuals,
        }
    }

    pub fn size(&self) -> usize {
        self.population.len()
    }

    pub fn add_individual(&mut self, ctx: &Context, individual: Individual) {
        // To always keep the vec sorted we find the insert index with a binary search
        let insert_index = match self.population.binary_search(&individual) {
            Ok(index) => index,
            Err(index) => index,
        };

        // Insert the indivual at the correct place
        self.population.insert(insert_index, individual);

        // Update the diversity in the population in relative to the new individual
        self.update_diversity(insert_index);

        if self.population.len() > 1 && self.individual_is_clone(&self.population[insert_index]) {
            // log::info!("Removing clone!");
            self.remove_individual(ctx, insert_index);
        } else {
            // Update the relative fitness in the population
            self.update_fitness(ctx);
        }

        // If the population has reached the max size, perform natural selection until the population
        // has min_population_size individuals left
        if self.population.len() >= self.max_individuals {
            let min_population_size = ctx.config.borrow().min_population_size as usize;
            while self.population.len() > min_population_size {
                self.natural_selection(ctx);
            }
        }
    }

    pub fn sample_top(&self, ctx: &Context, top: usize) -> (usize, &Individual) {
        let upper_limit = self.population.len().min(top);
        let index = ctx.random.range_usize(0, upper_limit);
        (index, &self.population[index])
    }

    /// Calculates the average diversity in the population
    pub fn get_diversity(&self, ctx: &Context) -> FloatType {
        // Only include the min_population_size best individuals
        let size = min(
            self.size(),
            ctx.config.borrow().min_population_size as usize,
        );

        let mut total = 0.0;
        for individual in &self.population[..size] {
            total += self.average_broken_pairs_distance(individual, size);
        }

        if size > 0 {
            total / size as FloatType
        } else {
            -1.0 as FloatType
        }
    }

    pub fn get_average_cost(&self, ctx: &Context) -> FloatType {
        // Only include the min_population_size best individuals
        let size = min(
            self.size(),
            ctx.config.borrow().min_population_size as usize,
        );

        let mut total = 0.0;
        for individual in &self.population[..size] {
            total += individual.penalized_cost();
        }

        if size > 0 {
            total / size as FloatType
        } else {
            -1.0 as FloatType
        }
    }

    pub fn get_best(&self) -> Option<&Individual> {
        if self.size() > 0 {
            Some(&self.population[0])
        } else {
            None
        }
    }

    pub fn get_best_cost(&self) -> FloatType {
        if let Some(best) = self.get_best() {
            best.penalized_cost()
        } else {
            0.0
        }
    }

    pub fn remove_individual(&mut self, ctx: &Context, index: usize) {
        // assert!(index > 0, "Removing best individual!");
        // Remove the individual from the population
        let individual = self.population.remove(index);

        // Remove the diversity between the individual and the rest of the population
        self.remove_diversity(&individual);

        // Update the relative fitness in the population
        self.update_fitness(ctx);
    }

    pub fn clear_worst(&mut self, ctx: &Context, keep: usize) {
        let mut index = self.population.len() - 1;
        while self.population.len() > keep {
            self.remove_individual(ctx, index);
            index -= 1;
        }
    }

    pub fn natural_selection(&mut self, ctx: &Context) {
        // Initialize the worst to the first individual
        let mut worst_index = 1usize;
        let mut worst_is_clone = false;
        let mut worst_fitness = -1.0;

        for index in 1..self.population.len() {
            let is_clone = self.individual_is_clone(&self.population[index]);

            // The worst should be updated if the worst is not a clone, but the individual is
            let mut update_worst = is_clone && !worst_is_clone;

            // Update worst based on fitness if either both are clones or none of them are clones
            update_worst = update_worst
                || (worst_is_clone == is_clone && self.population[index].fitness >= worst_fitness);

            // Update which individual is the worst
            if update_worst {
                worst_index = index;
                worst_is_clone = is_clone;
                worst_fitness = self.population[index].fitness;
            }
        }

        // let num_elites = ctx.config.borrow().num_elites as usize;

        // if !worst_is_clone && ctx.random.real() < 0.1 && self.population.len() >= num_elites {
        //     worst_index = ctx.random.range_usize(num_elites, self.population.len())
        // }

        self.remove_individual(ctx, worst_index);
    }

    fn individual_is_clone(&self, individual: &Individual) -> bool {
        // Check if the distance to the closest in the population is zero, then the individual
        // is considered to be a clone
        if let Some(diversity_vec) = self.diversity.get(&individual.number) {
            // Assumes that the diversity_vec of the individual is not empty
            diversity_vec[0].distance == 0
        } else {
            panic!("No diversity vector for indivdual");
        }
    }

    fn update_diversity(&mut self, index: usize) {
        // Calculate the diversity against all the other in the population
        for other_index in 0..self.population.len() {
            if index != other_index {
                let distance = self.population[other_index]
                    .calculate_broken_pairs_distance(&self.population[index]);
                self.add_diversity(
                    self.population[other_index].number,
                    Diversity::new(distance, self.population[index].number),
                );
                self.add_diversity(
                    self.population[index].number,
                    Diversity::new(distance, self.population[other_index].number),
                );
            }
        }
    }

    fn update_fitness(&mut self, ctx: &Context) {
        // Set the fitness to 0.0 (best) if the population only has one individual
        if self.population.len() == 1 {
            self.population[0].fitness = 0.0;
            return;
        }

        // if self.get_diversity(ctx) < 0.1 {
        //     self.focus_diversity = true;
        // } else {
        //     self.focus_diversity = false;
        // }

        let num_closest = ctx.config.borrow().num_diversity_closest as usize;

        // Vec used to sort the individuals after diversity in descending order
        let mut diversity_sorted: Vec<(FloatType, usize)> = self
            .population
            .iter()
            .enumerate()
            .map(|(index, individual)| {
                (
                    self.average_broken_pairs_distance(individual, num_closest),
                    index,
                )
            })
            .collect();

        // Sort the vec descending in diversity
        diversity_sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Precalculate shared factors
        let num_elites = ctx.config.borrow().num_elites as usize;
        let population_factor = self.population.len() as FloatType - 1.0;
        let elite_factor = 1.0 - num_elites as FloatType / self.population.len() as FloatType;

        // Calculate the biased fitness for each individual
        for (diversity_index, &(_, index)) in diversity_sorted.iter().enumerate() {
            // Ranks are normalized where 0 is the best and 1 is the worst
            let diversity_rank: FloatType = diversity_index as FloatType / population_factor;
            let fitness_rank: FloatType = index as FloatType / population_factor;

            if self.focus_diversity {
                let biased_fitness = if index < num_elites {
                    fitness_rank
                } else {
                    fitness_rank * elite_factor + diversity_rank
                };
                // Update the fitness of the individual
                self.population[index].fitness = biased_fitness;
            } else {
                // Calculate the biased fitness based on the two ranks
                let biased_fitness = if self.population.len() <= num_elites {
                    fitness_rank
                } else {
                    fitness_rank + elite_factor * diversity_rank
                };
                // Update the fitness of the individual
                self.population[index].fitness = biased_fitness;
            }
        }
    }

    fn add_diversity(&mut self, key: u64, diversity: Diversity) {
        // Get the diversity vector
        if let Some(diversity_vec) = self.diversity.get_mut(&key) {
            // Binary insert the diversity to preserve order
            let insert_index = match diversity_vec.binary_search(&diversity) {
                Ok(index) => index,
                Err(index) => index,
            };
            diversity_vec.insert(insert_index, diversity);
        } else {
            // If there is no vec, create it and insert the diversity
            let mut new_vec = Vec::with_capacity(self.max_individuals);
            new_vec.push(diversity);
            self.diversity.insert(key, new_vec);
        }
    }

    fn remove_diversity(&mut self, individual: &Individual) {
        // Remove the diversity vec for the number
        self.diversity.remove(&individual.number);

        // Remove the number from all the other vecs
        for (_, diversity_vec) in self.diversity.iter_mut() {
            for index in 0..diversity_vec.len() {
                if diversity_vec[index].to_number == individual.number {
                    diversity_vec.remove(index);
                    break;
                }
            }
        }
    }

    fn average_broken_pairs_distance(&self, individual: &Individual, num: usize) -> FloatType {
        // Check against the num_diversity_closest or the number of other individuals in the population
        let num_to_check = min(num, self.population.len() - 1);
        // Total sum of the diversity
        let mut diversity_total = 0;

        // Get reference to the diversity vec for the individual
        let diversity_vec = if let Some(div) = self.diversity.get(&individual.number) {
            div
        } else {
            return 0.0;
        };

        // Sum up the diversity agains the num_to_check closest in the population
        for index in 0..num_to_check {
            diversity_total += diversity_vec[index].distance;
        }

        // Return the average
        return diversity_total as FloatType / num_to_check as FloatType;
    }
}

pub struct Population {
    // Total number of individuals that has been a part of the population
    pub total_individuals_count: u64,

    // Feasible subpopulation
    pub feasible: SubPopulation,
    // Infeasible subpopulation
    pub infeasible: SubPopulation,

    // History of the feasibility of the individuals added to the population
    pub feasible_history: Vec<bool>,
}

impl Population {
    pub fn new(ctx: &Context) -> Self {
        Self {
            total_individuals_count: 0,
            feasible: SubPopulation::new(ctx),
            infeasible: SubPopulation::new(ctx),
            feasible_history: vec![true; 100],
        }
    }

    pub fn size(&self) -> usize {
        self.feasible.size() + self.infeasible.size()
    }

    pub fn add_individual(
        &mut self,
        ctx: &Context,
        mut individual: Individual,
        update_feasibility_history: bool,
    ) {
        individual.number = self.total_individuals_count;
        if update_feasibility_history {
            self.feasible_history.push(individual.is_feasible());
            self.feasible_history.remove(0);
        }
        if individual.is_feasible() {
            self.feasible.add_individual(ctx, individual);
        } else {
            self.infeasible.add_individual(ctx, individual);
        };

        // Increment total individuals count
        self.total_individuals_count += 1;
    }

    pub fn get_parent(&self, ctx: &Context) -> &Individual {
        self.tournament(ctx, ctx.config.borrow().tournament_size as usize)
    }

    pub fn history_fraction(&self) -> FloatType {
        self.feasible_history.iter().filter(|&&x| x).count() as FloatType
            / self.feasible_history.len() as FloatType
    }

    fn tournament(&self, ctx: &Context, num_contestants: usize) -> &Individual {
        // let lower = if ctx.random.real() < 0.1 { 0 } else { 1 };
        let lower = 0;
        // Sample `k` individuals from the two subpopulations
        let indicies: Vec<usize> = (0..num_contestants)
            .into_iter()
            .map(|_| ctx.random.range_usize(lower, self.size()))
            .collect();

        // Select the winner which has the lowest cost
        let mut winner: Option<&Individual> = None;
        for index in indicies {
            // Select the correct subpopulation and offset the index
            let (sub_population, population_index) = if index < self.feasible.size() {
                (&self.feasible, index)
            } else {
                (&self.infeasible, index - self.feasible.size())
            };

            // Update the winner if the individual is better
            let individual = &sub_population.population[population_index];
            if let Some(current_winner) = winner {
                if individual.fitness < current_winner.fitness {
                    winner = Some(individual);
                }
            } else {
                winner = Some(individual);
            }
        }

        winner.expect("No winner found")
    }
}
