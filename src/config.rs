use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::cli::Args;
use crate::models::FloatType;

/// Contains all the configuration parameters
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // General
    pub instance_path: String,
    pub solution_path: String,
    pub time_limit: u64,
    pub max_iterations: u64,
    pub num_vehicles: u64,
    pub log_interval: u64,

    // Randomization
    pub deterministic: bool,
    pub seed: u64,

    // Genetic Algorithm
    pub min_population_size: u64,
    pub initial_individuals: u64,
    pub population_lambda: u64,
    pub num_elites: u64,
    pub num_diversity_closest: u64,
    pub feasibility_proportion_target: FloatType,
    pub tournament_size: u64,
    pub repair_probability: FloatType,

    // Split
    pub split_capacity_factor: FloatType,
    pub linear_split: bool,

    // Local Search
    pub local_search_granularity: u64,
    pub dynamic_granularity: bool,
    pub granularity_min: u64,
    pub ls_enabled: bool,

    // Local Search Moves
    pub relocate_single: bool,
    pub relocate_double: bool,
    pub relocate_double_reverse: bool,
    pub swap_one_with_one: bool,
    pub swap_two_with_one: bool,
    pub swap_two_with_two: bool,
    pub two_opt_intra_reverse: bool,
    pub two_opt_inter_reverse: bool,
    pub two_opt_inter: bool,
    pub swap_star: bool,

    // Penalties
    pub penalty_capacity: FloatType,
    pub penalty_update_interval: u64,
    pub penalty_inc_multiplier: FloatType,
    pub penalty_dec_multiplier: FloatType,

    // Ruin Recreate
    pub average_ruin_cardinality: usize,
    pub max_ruin_string_length: usize,
    pub rr_mutation: bool,
    pub rr_probability: FloatType,
    pub rr_gamma: FloatType,
    pub rr_final_temp: FloatType,
    pub rr_start_temp: FloatType,
    pub rr_diversify: bool,

    // Diving with ruin recreate
    pub elite_education: bool,
    pub elite_education_problem_size_limit: usize,
    pub elite_education_gamma: FloatType,
    pub elite_education_final_temp: FloatType,
    pub elite_education_start_temp: FloatType,
}

impl Config {
    pub fn default() -> Self {
        Self {
            // General
            instance_path: String::new(),
            solution_path: String::new(),
            time_limit: 60,
            max_iterations: 20_000,
            num_vehicles: 1_000_000,
            log_interval: 100,

            // Randomization
            deterministic: false,
            seed: 1,

            // Genetic Algorithm
            min_population_size: 25,
            initial_individuals: 100,
            population_lambda: 40,
            num_elites: 4,
            num_diversity_closest: 5,
            feasibility_proportion_target: 0.2,
            tournament_size: 2,
            repair_probability: 0.5,

            // Split
            split_capacity_factor: 1.5,
            linear_split: true,

            // Local Search
            ls_enabled: true,
            local_search_granularity: 20,
            dynamic_granularity: false,
            granularity_min: 10,

            // Local Search Moves
            relocate_single: true,
            relocate_double: true,
            relocate_double_reverse: false,
            swap_one_with_one: true,
            swap_two_with_one: true,
            swap_two_with_two: true,
            two_opt_intra_reverse: true,
            two_opt_inter_reverse: true,
            two_opt_inter: true,
            swap_star: true,

            // Penalties
            penalty_capacity: 100.0,
            penalty_dec_multiplier: 0.85,
            penalty_inc_multiplier: 1.2,
            penalty_update_interval: 10,

            // Ruin Recreate
            average_ruin_cardinality: 10,
            max_ruin_string_length: 10,
            rr_mutation: true,
            rr_gamma: 1.0,
            rr_probability: 1.0,
            rr_final_temp: 1.0,
            rr_start_temp: 10.0,
            rr_diversify: true,

            // Diving with ruin recreate
            elite_education: false,
            elite_education_problem_size_limit: 1,
            elite_education_gamma: 1_000.0,
            elite_education_final_temp: 1.0,
            elite_education_start_temp: 50.0,
        }
    }

    /// Reset the config to default values
    pub fn reset(&mut self) {
        let new_config = Self::default();
        *self = new_config;
    }

    fn read_yaml_file(filepath: &str) -> Value {
        let file = File::open(filepath).expect(&format!("Cannot open file {}", filepath));
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).expect(&format!("Failed to read file {}", filepath))
    }

    /// Load config from yaml file
    pub fn load_yaml_file(filepath: &str) -> Self {
        // Load default
        let mut config = Self::default();

        // Update default with loaded values
        config.update(&Self::read_yaml_file(filepath));
        config
    }

    /// Update the config with values from a YAML file
    pub fn update_from_yaml_file(&mut self, filepath: &str) {
        self.update(&Self::read_yaml_file(filepath));
    }

    /// Update the config with YAML values
    pub fn update(&mut self, values: &Value) {
        let mut config: Value = serde_yaml::to_value(&self).expect("Failed to serialize config");
        match values {
            Value::Mapping(mapping) => {
                // Iterate over all key-value pairs in the mapping and update the config
                for (key, value) in mapping.iter() {
                    if let Some(config_value) = config.get_mut(key) {
                        *config_value = value.clone();
                    }
                }
            }
            _ => panic!("Cannot update Config as YAML is not a mapping"),
        }
        // Update the config object
        *self = serde_yaml::from_value(config).expect("Failed to deserialize patched config");
    }

    /// Update config with command line arguments
    pub fn update_from_args(&mut self, args: &Args) {
        self.instance_path = args.instance_path.clone();
        self.solution_path = args.solution_path.clone();
        if let Some(max_iterations) = args.max_iterations {
            self.max_iterations = max_iterations;
        }
        if let Some(time_limit) = args.time_limit {
            self.time_limit = time_limit;
        }
    }
}
