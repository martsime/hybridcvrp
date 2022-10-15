use clap::{App, Arg};

/// Representing the resulting command line arguments
pub struct Args {
    pub instance_path: String,
    pub solution_path: Option<String>,
    pub time_limit: Option<u64>,
    pub max_iterations: Option<u64>,
    pub rounded: bool,
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
                    .default_value("output.sol")
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
                    .default_value("10")
                    .help("Time limit in seconds"),
            )
            .arg(
                Arg::with_name("rounded")
                    .short("r")
                    .takes_value(true)
                    .default_value("true")
                    .help("Rounded distances"),
            )
            .get_matches();

        let instance_path = matches
            .value_of("instance_path")
            .expect("Instance path is not provided")
            .to_owned();

        let solution_path = matches.value_of("solution_path").map(String::from);

        let max_iterations = matches
            .value_of("iterations")
            .map(|value| value.parse::<u64>().expect("Invalid iterations argument!"));

        let time_limit = matches
            .value_of("time_limit")
            .map(|value| value.parse::<u64>().expect("Invalid time limit argument!"));

        let rounded = matches.is_present("rounded");

        Self {
            instance_path,
            solution_path,
            time_limit,
            max_iterations,
            rounded,
        }
    }
}
