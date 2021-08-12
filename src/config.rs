use std::fs::File;
use std::io::BufReader;

use crate::models::FloatType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // General
    pub problem_instance: String,
    pub dataset: String,
    pub time_limit: u64,
    pub max_iterations_without_improvement: u64,
    pub max_iterations: Option<u64>,
    pub num_vehicles: u64,
    pub run_id: Option<Uuid>,
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
    pub diversify_after_iterations: Option<u64>,

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
    pub rr_threshold_switch: FloatType,
    pub rr_threshold_one: FloatType,
    pub rr_threshold_two: FloatType,
    pub rr_final_temp: FloatType,
    pub rr_start_temp: FloatType,
    pub rr_diversify: bool,
    pub rr_acceptance_alpha: FloatType,

    // Diving with ruin recreate
    pub elite_education: bool,
    pub dive_problem_size_limit: usize,
    pub elite_education_gamma: usize,
    pub dive_threshold_switch: FloatType,
    pub dive_threshold_one: FloatType,
    pub dive_threshold_two: FloatType,
    pub elite_education_final_temp: FloatType,
    pub elite_education_start_temp: FloatType,
}

impl Config {
    pub fn default() -> Self {
        Self {
            // General
            problem_instance: String::new(),
            dataset: String::new(),
            time_limit: 60,
            max_iterations_without_improvement: 20_000,
            max_iterations: None,
            num_vehicles: 1_000_000,
            run_id: None,
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
            diversify_after_iterations: None,

            // Split
            split_capacity_factor: 1.5,
            linear_split: true,

            // Local Search
            local_search_granularity: 20,
            dynamic_granularity: false,
            granularity_min: 10,
            ls_enabled: true,

            // Local Search Moves
            relocate_single: true,
            relocate_double: true,
            relocate_double_reverse: true,
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
            rr_threshold_switch: 1.0,
            rr_threshold_one: 2.0,
            rr_threshold_two: 2.0,
            rr_diversify: true,
            rr_acceptance_alpha: 1.0,

            // Diving with ruin recreate
            elite_education: true,
            dive_problem_size_limit: 1,
            elite_education_gamma: 10_000,
            dive_threshold_switch: 0.0,
            dive_threshold_one: 1.1,
            dive_threshold_two: 1.1,
            elite_education_final_temp: 1.0,
            elite_education_start_temp: 50.0,
        }
    }

    pub fn reset(&mut self) {
        let new_config = Self::default();
        *self = new_config;
    }

    fn read_yaml_file(filepath: &str) -> Value {
        let file = File::open(filepath).expect(&format!("Cannot open file {}", filepath));
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).expect(&format!("Failed to read file {}", filepath))
    }

    pub fn load_yaml_file(filepath: &str) -> Self {
        // Load default
        let mut config = Self::default();

        // Patch default with loaded values
        config.patch(&Self::read_yaml_file(filepath));
        config
    }

    pub fn patch_from_yaml_file(&mut self, filepath: &str) {
        self.patch(&Self::read_yaml_file(filepath));
    }

    pub fn patch(&mut self, values: &Value) {
        let mut config: Value = serde_json::to_value(&self).expect("Failed to serialize config");
        match values {
            Value::Object(values_map) => {
                // Iterate over all key-value pairs in the provided values and update the config
                for (key, value) in values_map.iter() {
                    // The key is like a file path. A key at top level starts with /
                    let root_key = format!("/{}", key);
                    if let Some(config_value) = config.pointer_mut(&root_key) {
                        *config_value = value.clone();
                    }
                }
            }
            _ => panic!("Cannot patch Config as JSON is not an Object"),
        }
        // Update the config object
        *self = serde_json::from_value(config).expect("Failed to deserialize patched config");
    }
}
