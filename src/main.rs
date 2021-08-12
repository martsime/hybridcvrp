use std::time::Instant;

use hybridcvrp::config::Config;
use hybridcvrp::solver::genetic::GeneticAlgorithm;
use hybridcvrp::solver::{Context, SearchHistory, Solver};
use hybridcvrp::utils;

fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .format_module_path(false)
        .init();
    let start_time = Instant::now();
    let mut search_history = SearchHistory::new(start_time);

    let mut config = Config::default();
    config.patch_from_yaml_file("config.yml");

    let problem = utils::parse_problem(&mut config);
    search_history.add_message(format!(
        "Loading problem {} complete",
        config.problem_instance
    ));
    let ctx = Context::new(problem, config);
    let metaheuristic = GeneticAlgorithm::new(&ctx, search_history);
    let mut solver = Solver::new(ctx, metaheuristic);
    solver.start();
}
