use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::time::{Duration, Instant};

use hybridcvrp::config::Config;
use hybridcvrp::solver::genetic::GeneticAlgorithm;
use hybridcvrp::solver::{Context, Solver};
use hybridcvrp::utils;

fn code(_lol: usize) {
    let start_time = Instant::now();
    let mut config = Config::load_yaml_file("config.yml");
    config.deterministic = true;
    config.max_iterations = Some(500);
    config.time_limit = 1000;
    config.instance_path = "instances/X-n101-k25.vrp".to_owned();
    config.elite_education = false;

    let mut parser = utils::ProblemParser::new();
    parser.parse(&mut config);
    let ctx = Context::new(parser, config, start_time);
    let metaheuristic = GeneticAlgorithm::new(&ctx);
    let mut solver = Solver::new(ctx, metaheuristic);
    solver.run();
}

pub fn bench(c: &mut Criterion) {
    c.bench_function("Metaheuristic", |b| b.iter(|| code(black_box(0))));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50).measurement_time(Duration::from_secs(60));
    targets = bench
}
criterion_main!(benches);
