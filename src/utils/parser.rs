use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::config::Config;
use crate::models::IntType;
use crate::models::{Coordinate, Node, Problem, ProblemBuilder, Vehicle};

pub fn parse_problem(config: &mut Config) -> Problem {
    let filepath = Path::new(&config.instance_path);
    assert!(filepath.exists(), "Cannot find instance file");

    let file = File::open(&config.instance_path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let line_strings: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();

    let lines: Vec<Vec<String>> = line_strings
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
        .collect();

    let mut line_number = 0;
    let dimensions = {
        let value;
        loop {
            if lines[line_number][0] == "DIMENSION" {
                value = lines[line_number][1]
                    .parse::<usize>()
                    .expect("Failed to parse dimension");
                break;
            } else {
                line_number += 1;
            }
        }
        value
    };
    line_number = 0;
    let capacity = {
        let value;
        loop {
            if lines[line_number][0] == "CAPACITY" {
                value = lines[line_number][1]
                    .parse::<IntType>()
                    .expect("Failed to parse capacity");
                break;
            } else {
                line_number += 1;
            }
        }
        value
    };

    line_number = 0;
    let coordinates = {
        let mut coords = Vec::new();
        loop {
            if lines[line_number][0] == "NODE_COORD_SECTION" {
                break;
            } else {
                line_number += 1;
            }
        }

        line_number += 1;
        for i in 0..dimensions {
            coords.push(Coordinate {
                lng: lines[line_number + i][1]
                    .parse::<IntType>()
                    .expect("Failed to parse coordinate"),
                lat: lines[line_number + i][2]
                    .parse::<IntType>()
                    .expect("Failed to parse coordinate"),
            });
        }
        coords
    };

    line_number = 0;
    let demands = {
        let mut demand_values = Vec::new();
        loop {
            if lines[line_number][0] == "DEMAND_SECTION" {
                break;
            } else {
                line_number += 1;
            }
        }

        line_number += 1;
        for i in 0..dimensions {
            demand_values.push(
                lines[line_number + i][1]
                    .parse::<IntType>()
                    .expect("Failed to parse demand"),
            );
        }
        demand_values
    };

    let nodes: Vec<Node> = (0..dimensions)
        .into_iter()
        .map(|i| Node {
            id: i + 1,
            coord: Coordinate {
                lat: coordinates[i].lat,
                lng: coordinates[i].lng,
            },
            demand: demands[i],
        })
        .collect();
    let vehicle = Vehicle {
        id: 0,
        cap: capacity,
    };

    let problem_builder = ProblemBuilder::new(nodes, vehicle);
    let problem = problem_builder.build(config);
    problem
}
