use std::cmp::{min, Reverse};
use std::collections::BinaryHeap;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::models::{FloatType, IntType, Matrix};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Coordinate {
    pub lng: IntType,
    pub lat: IntType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: usize,
    pub coord: Coordinate,
    pub demand: IntType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vehicle {
    pub id: usize,
    pub cap: IntType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemBuilder {
    pub nodes: Vec<Node>,
    pub vehicle: Vehicle,

    // Distances stored as a single dimensional vec
    #[serde(skip)]
    distances: Option<Matrix<IntType>>,

    neighbors: Option<Vec<usize>>,

    max_distance: Option<IntType>,
    max_demand: Option<IntType>,

    // Minimum number of vehicles from the LP bin packing problem
    vehicle_lower_bound: Option<u64>,

    // Distances stored as a single dimensional vec
    correlations: Option<Vec<usize>>,
    granularities: Option<Vec<usize>>,

    // Number of correlations per node
    num_correlations: Option<usize>,
}

impl ProblemBuilder {
    pub fn new(nodes: Vec<Node>, vehicle: Vehicle) -> Self {
        Self {
            nodes,
            vehicle,
            distances: None,
            neighbors: None,
            max_distance: None,
            max_demand: None,
            vehicle_lower_bound: None,
            correlations: None,
            granularities: None,
            num_correlations: None,
        }
    }

    pub fn build(mut self, config: &mut Config) -> Problem {
        self.generate_distances();
        self.generate_neighbors();
        self.generate_correlations(config);
        self.calculate_vehicles_lower_bound();
        config.num_vehicles = self.calculate_initial_num_vehicles();
        config.penalty_capacity = self.calculate_penalty_capacity();
        Problem {
            nodes: self.nodes,
            vehicle: self.vehicle,
            distance: self.distances.expect("Cannot build, no distances"),
            neighbors: self.neighbors.expect("Cannot build, no neighbors"),
            vehicle_lower_bound: self
                .vehicle_lower_bound
                .expect("Cannot build, no vehicle_lower_bound"),
            correlations: self.correlations.expect("Cannot build, no correlations"),
            granularities: self.granularities.expect("Cannot build, no granularities"),
            num_correlations: self
                .num_correlations
                .expect("Cannot build, no num_correlations"),
        }
    }

    pub fn generate_neighbors(&mut self) {
        let distances = self.distances.as_ref().expect("No distances");
        let num_nodes = self.nodes.len();
        let mut neighbors = vec![0; num_nodes * (num_nodes - 1)];
        for i in 1..num_nodes {
            let mut neighbors_i: Vec<(usize, IntType)> = distances
                .slice(i, 1, num_nodes - 1)
                .iter()
                .enumerate()
                .map(|(j, &distance)| (j + 1, distance))
                .collect();
            neighbors_i.sort_by(|a, b| a.1.cmp(&b.1));
            for (index, &(neighbor_index, _)) in neighbors_i.iter().enumerate() {
                neighbors[i * (num_nodes - 1) + index] = neighbor_index;
            }
        }
        self.neighbors = Some(neighbors);
    }

    pub fn generate_distances(&mut self) {
        let num_nodes = self.nodes.len();
        let nodes = &self.nodes;
        let mut distances = Matrix::new(num_nodes, num_nodes);
        let mut max_distance = 0;
        let mut max_demand = 0;
        for i in 0..num_nodes {
            if self.nodes[i].demand > max_demand {
                max_demand = self.nodes[i].demand;
            }
            for j in (i + 1)..num_nodes {
                let distance = ((nodes[j].coord.lng as FloatType - nodes[i].coord.lng as FloatType)
                    .powi(2)
                    + (nodes[j].coord.lat as FloatType - nodes[i].coord.lat as FloatType).powi(2))
                .sqrt()
                .round() as IntType;
                if distance > max_distance {
                    max_distance = distance;
                }

                // The matrix is symmetric
                distances.set(i, j, distance);
                distances.set(j, i, distance);
            }
        }
        self.max_distance = Some(max_distance);
        self.max_demand = Some(max_demand);
        // println!("{:?}, {:?}", self.max_distance, self.max_demand);
        self.distances = Some(distances);
    }

    fn calculate_vehicles_lower_bound(&mut self) {
        // Simple bin packing problem
        let total_demand: IntType = self.nodes.iter().map(|node| node.demand).sum();
        let lower_bound = (total_demand as FloatType / self.vehicle.cap as FloatType).ceil() as u64;
        self.vehicle_lower_bound = Some(lower_bound);
    }

    fn calculate_initial_num_vehicles(&self) -> u64 {
        // Safety maring: 20% + 2 more than the bin packing lower bound
        let lower_bound = self
            .vehicle_lower_bound
            .expect("Vehicle lower bound is not calculated") as FloatType;
        (1.2 as FloatType * lower_bound + 2.0).ceil() as u64
    }

    fn calculate_penalty_capacity(&mut self) -> FloatType {
        let max_distance = self.max_distance.expect("No max distance") as FloatType;
        let max_demand = self.max_demand.expect("No max demand") as FloatType;
        (0.0001 as FloatType).max((10_000 as FloatType).min(max_distance / max_demand))
    }

    pub fn distance(&self, from: usize, to: usize) -> IntType {
        if let Some(distances) = self.distances.as_ref() {
            distances.get(from, to)
        } else {
            panic!("Distances not generated");
        }
    }

    pub fn generate_correlations(&mut self, config: &mut Config) {
        let num_nodes = self.nodes.len();
        let target_granularity = config.local_search_granularity;

        // The max number of correleations is either limited by the min of the config and the number of nodes in the problem
        let num_correlations = min(2 * target_granularity as usize, num_nodes - 2);
        self.num_correlations = Some(num_correlations);
        let mut granularities: Vec<usize> = Vec::with_capacity(num_nodes);

        let mut total_distance = 0;

        let mut correlations = vec![0; num_nodes * num_correlations];
        for node_number in 0..num_nodes {
            // Create a min binary heap with the other nodes ordered on distance
            let mut distances_heap: BinaryHeap<Reverse<(IntType, usize)>> = (0..num_nodes)
                .into_iter()
                .filter_map(|to_node| {
                    // Don't calculate correlation to self
                    if node_number == to_node || to_node == 0 {
                        None
                    } else {
                        Some(Reverse((self.distance(node_number, to_node), to_node)))
                    }
                })
                .collect();

            // Collect the `num_correlations` closest nodes from the binary heap
            let node_distances: Vec<(IntType, usize)> = (0..num_correlations)
                .into_iter()
                .map(|_| distances_heap.pop().expect("No element in the heap").0)
                .collect();

            // Update the nodes in the correlation vector
            for (node_index, &(distance, correlated_node)) in node_distances.iter().enumerate() {
                correlations[node_number * num_correlations + node_index] = correlated_node;
                total_distance += distance;
            }
        }

        let mut mean_distance = total_distance as f64 / (num_nodes * num_correlations) as f64;

        if config.dynamic_granularity {
            loop {
                for node_index in 0..num_nodes {
                    let start_index = node_index * num_correlations;
                    let neighbors = &correlations[start_index..(start_index + num_correlations)];
                    let mut granularity = 0;
                    for &neighbor in neighbors.iter() {
                        if self.distance(node_index, neighbor) as f64 <= mean_distance {
                            granularity += 1;
                        } else {
                            break;
                        }
                    }
                    granularities.push((config.granularity_min as usize).max(granularity))
                }
                let average_granularity = (granularities.iter().map(|g| *g as f64).sum::<f64>()
                    / granularities.len() as f64)
                    .round() as u64;
                if average_granularity < target_granularity {
                    mean_distance *= 1.1f64;
                } else if average_granularity > target_granularity {
                    mean_distance *= 0.9f64;
                } else {
                    break;
                }

                granularities.clear();
            }
        } else {
            for _ in 0..num_nodes {
                granularities.push(target_granularity as usize);
            }
        }
        self.granularities = Some(granularities);
        self.correlations = Some(correlations);
    }
}

#[derive(Debug)]
pub struct Problem {
    pub nodes: Vec<Node>,
    pub vehicle: Vehicle,

    // Distances stored as a single dimensional vec
    pub distance: Matrix<IntType>,
    neighbors: Vec<usize>,

    // Lower bound on number of vehicles
    pub vehicle_lower_bound: u64,

    // Distances stored as a single dimensional vec
    pub correlations: Vec<usize>,

    // Number of neighbours for each node
    pub granularities: Vec<usize>,

    // Number of correlations per node
    pub num_correlations: usize,
}

impl Problem {
    pub fn neighbors(&self, node: usize) -> &[usize] {
        let start_index = (self.nodes.len() - 1) * node;
        let end_index = start_index + self.nodes.len() - 1;
        if cfg!(feature = "unsafe-speedup") {
            unsafe { self.neighbors.get_unchecked(start_index..end_index) }
        } else {
            &self.neighbors[start_index..end_index]
        }
    }

    pub fn correlations(&self, node: usize) -> &[usize] {
        // Start and end of the correlations slice
        let start_index = self.num_correlations * node;
        let end_index = start_index + self.num_correlations;

        if cfg!(feature = "unsafe-speedup") {
            unsafe { self.correlations.get_unchecked(start_index..end_index) }
        } else {
            &self.correlations[start_index..end_index]
        }
    }

    // Dimension of the problem
    pub fn dim(&self) -> usize {
        self.nodes.len()
    }

    // Number of customer nodes in the problem
    pub fn num_customers(&self) -> usize {
        self.dim() - 1
    }

    pub fn get_angle(&self, node: usize) -> IntType {
        let x = self.nodes[node].coord.lng as FloatType - self.nodes[0].coord.lng as FloatType;
        let y = self.nodes[node].coord.lat as FloatType - self.nodes[0].coord.lat as FloatType;
        let angle = (((y.atan2(x) / std::f64::consts::PI as FloatType) * 32768.0).round()
            as IntType)
            .rem_euclid(65536);
        angle
    }
}
