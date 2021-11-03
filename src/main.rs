use instant::Instant;

use hybridcvrp::cli::Args;
use hybridcvrp::config::Config;
use hybridcvrp::solver::genetic::GeneticAlgorithm;
use hybridcvrp::solver::{Context, Solver};
use hybridcvrp::utils;

fn main() {
    // Start time of program
    let start_time = Instant::now();

    // Initialize logger
    env_logger::Builder::from_default_env()
        .format_module_path(false)
        .init();

    // Load config
    let mut config = Config::load_yaml_file("config.yml");
    log::info!("Loading config");

    // Parse command line arguments
    let args = Args::parse();
    config.update_from_args(&args);

    log::info!("Loading problem file: {}", config.instance_path);
    let problem = utils::parse_problem(&mut config);
    log::info!("Problem load complete");

    let ctx = Context::new(problem, config, start_time);
    let metaheuristic = GeneticAlgorithm::new(&ctx);
    let mut solver = Solver::new(ctx, metaheuristic);
    solver.run();
    utils::write_solution_file(&solver.ctx);
}
