use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::config::Config;
use crate::models::{Coordinate, Node, Problem, ProblemBuilder, Vehicle};

type Lines = Vec<Vec<String>>;

enum EdgeWeightType {
    Euclidian2D,
    Explicit,
}

impl FromStr for EdgeWeightType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EUC_2D" => Ok(Self::Euclidian2D),
            "EXPLICIT" => Ok(Self::Explicit),
            _ => Err(format!("Unknown EDGE_WEIGHT_TYPE: {}", s)),
        }
    }
}

enum EdgeWeightFormat {
    LowerRow,
}

impl FromStr for EdgeWeightFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOWER_ROW" => Ok(Self::LowerRow),
            _ => Err(format!("Unknown EDGE_WEIGHT_FORMAT: {}", s)),
        }
    }
}

pub struct ProblemParser {
    pub problem: Option<Problem>,
    pub matrix: Option<Vec<Vec<f64>>>,
}

impl ProblemParser {
    pub fn new() -> Self {
        Self {
            problem: None,
            matrix: None,
        }
    }

    pub fn parse(&mut self, config: &mut Config) {
        let filepath = Path::new(&config.instance_path);
        assert!(filepath.exists(), "Cannot find instance file");

        let lines = Self::read_file(&filepath);

        let dimension = Self::parse_dimension(&lines);
        let capacity = Self::parse_capacity(&lines);
        let coords = Self::parse_coords(&lines, dimension);
        let demands = Self::parse_demands(&lines, dimension);
        let nodes = Self::create_nodes(coords, demands);
        let vehicle = Self::create_vehicle(0, capacity);

        let problem_builder = ProblemBuilder::new(nodes, vehicle);
        let problem = problem_builder.build();
        self.problem = Some(problem);

        match Self::parse_edge_weight_type(&lines) {
            EdgeWeightType::Euclidian2D => {}
            EdgeWeightType::Explicit => match Self::parse_edge_weight_format(&lines) {
                EdgeWeightFormat::LowerRow => {
                    let matrix = Self::parse_lower_row_matrix(&lines, dimension);
                    self.matrix = Some(matrix);
                }
            },
        };
    }

    fn read_file(path: &Path) -> Lines {
        let file = File::open(path).expect("Failed to open file");
        let reader = BufReader::new(file);
        let line_strings: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();

        line_strings
            .iter()
            .map(|line| {
                let line_values: Vec<String> = line
                    .split(&[' ', '\t', ':'][..])
                    .filter_map(|value| match value {
                        "" => None,
                        _ => Some(value.to_owned()),
                    })
                    .collect();
                line_values
            })
            .collect()
    }

    fn parse_dimension(lines: &Lines) -> usize {
        for line in lines.iter() {
            if !line.is_empty() && line[0] == "DIMENSION" {
                return line[1].parse::<usize>().expect("Failed to parse dimension");
            }
        }
        panic!("Could not find DIMENSION");
    }

    fn parse_capacity(lines: &Lines) -> f64 {
        for line in lines.iter() {
            if !line.is_empty() && line[0] == "CAPACITY" {
                return line[1].parse::<f64>().expect("Failed to parse capacity");
            }
        }
        panic!("Could not find CAPACITY");
    }

    fn parse_edge_weight_type(lines: &Lines) -> EdgeWeightType {
        for line in lines.iter() {
            if !line.is_empty() && line[0] == "EDGE_WEIGHT_TYPE" {
                return match EdgeWeightType::from_str(&line[1]) {
                    Ok(edge_type) => edge_type,
                    Err(err) => {
                        panic!("{}", err);
                    }
                };
            }
        }
        panic!("Could not find EDGE_WEIGHT_TYPE");
    }

    fn parse_edge_weight_format(lines: &Lines) -> EdgeWeightFormat {
        for line in lines.iter() {
            if !line.is_empty() && line[0] == "EDGE_WEIGHT_FORMAT" {
                return match EdgeWeightFormat::from_str(&line[1]) {
                    Ok(edge_format) => edge_format,
                    Err(err) => {
                        panic!("{}", err);
                    }
                };
            }
        }
        panic!("Could not find EDGE_WEIGHT_FORMAT");
    }

    fn parse_coords(lines: &Lines, number: usize) -> Vec<Coordinate> {
        for (line_number, line) in lines.iter().enumerate() {
            if !line.is_empty() && line[0] == "NODE_COORD_SECTION" {
                return lines
                    .iter()
                    .skip(line_number + 1)
                    .take(number)
                    .map(|line| Coordinate {
                        lng: line[1].parse::<f64>().expect("Failed to parse coordinate"),
                        lat: line[2].parse::<f64>().expect("Failed to parse coordinate"),
                    })
                    .collect();
            }
        }
        panic!("Could not find NODE_COORD_SECTION");
    }

    fn parse_demands(lines: &Lines, number: usize) -> Vec<f64> {
        for (line_number, line) in lines.iter().enumerate() {
            if !line.is_empty() && line[0] == "DEMAND_SECTION" {
                return lines
                    .iter()
                    .skip(line_number + 1)
                    .take(number)
                    .map(|line| line[1].parse::<f64>().expect("Failed to parse demand"))
                    .collect();
            }
        }
        panic!("Could not find DEMAND_SECTION");
    }

    fn parse_lower_row_matrix(lines: &Lines, number: usize) -> Vec<Vec<f64>> {
        for (line_number, line) in lines.iter().enumerate() {
            if !line.is_empty() && line[0] == "EDGE_WEIGHT_SECTION" {
                return lines
                    .iter()
                    .skip(line_number + 1)
                    .take(number - 1)
                    .map(|line| {
                        line.iter()
                            .map(|val| val.parse::<f64>().expect("Failed to parse edge weight"))
                            .collect()
                    })
                    .collect();
            }
        }
        panic!("Could not find EDGE_WEIGHT_SECTION");
    }

    fn create_nodes(coords: Vec<Coordinate>, demands: Vec<f64>) -> Vec<Node> {
        coords
            .into_iter()
            .zip(demands.into_iter())
            .enumerate()
            .map(|(i, (coord, demand))| Node {
                id: i + 1,
                coord,
                demand,
            })
            .collect()
    }

    fn create_vehicle(id: usize, capacity: f64) -> Vehicle {
        Vehicle { id, cap: capacity }
    }
}
