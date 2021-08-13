use clap::{App, Arg};

/// Representing the resulting command line arguments
pub struct Args {
    pub instance_path: String,
    pub solution_path: String,
    pub time_limit: Option<u64>,
    pub max_iterations: Option<u64>,
}

impl Args {
    /// Setup the clap app and parse the command line arguments
    pub fn parse() -> Self {
        let matches = App::new("hybridcvrp")
            .version("0.1")
            .author("Martin Simensen")
            .about("Hybrid Metaheuristic Solver for the Capacitated Vehicle Routing Problem")
            .arg(
                Arg::with_name("instance_path")
                    .required(true)
                    .help("Path to problem instance"),
            )
            .arg(
                Arg::with_name("solution_path")
                    .short("o")
                    .takes_value(true)
                    .help("Path to solution output"),
            )
            .arg(
                Arg::with_name("iterations")
                    .short("i")
                    .takes_value(true)
                    .help("Maximum number of iterations without improvement"),
            )
            .arg(
                Arg::with_name("time_limit")
                    .short("t")
                    .takes_value(true)
                    .help("Time limit in seconds"),
            )
            .get_matches();

        let instance_path = matches
            .value_of("instance_path")
            .expect("Instance path is not provided")
            .to_owned();

        let solution_path = matches
            .value_of("solution_path")
            .unwrap_or("output.sol")
            .to_owned();

        let max_iterations = if let Some(iterations) = matches.value_of("iterations") {
            Some(
                iterations
                    .parse::<u64>()
                    .expect("Invalid iterations argument"),
            )
        } else {
            None
        };

        let time_limit = if let Some(time_limit) = matches.value_of("time_limit") {
            Some(
                time_limit
                    .parse::<u64>()
                    .expect("Invalid time limit argument"),
            )
        } else {
            None
        };

        Self {
            instance_path,
            solution_path,
            time_limit,
            max_iterations,
        }
    }
}
