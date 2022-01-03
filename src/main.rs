use instant::Instant;

use hybridcvrp::cli::Args;
use hybridcvrp::config::Config;
use hybridcvrp::solver::genetic::{DecomposedGeneticAlgorithm, GeneticAlgorithm};
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
    let mut parser = utils::ProblemParser::new();
    parser.parse(&mut config);
    let ctx = Context::new(parser, config, start_time);
    log::info!("Problem load complete");

    let should_decompose =
        ctx.problem.num_customers() as u64 >= ctx.config.borrow().decompose_limit;

    match should_decompose {
        true => {
            let metaheuristic = DecomposedGeneticAlgorithm::new(&ctx);
            let mut solver = Solver::new(ctx, metaheuristic);
            log::info!("Algorithm: Decomposed Genetic Algorithm");
            solver.run();
            utils::write_solution_file(&solver.ctx);
        }
        false => {
            let metaheuristic = GeneticAlgorithm::new(&ctx);
            let mut solver = Solver::new(ctx, metaheuristic);
            log::info!("Algorithm: Genetic Algorithm");
            solver.run();
            utils::write_solution_file(&solver.ctx);
        }
    };
}
