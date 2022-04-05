use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Coordinate {
    pub lng: f64,
    pub lat: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: usize,
    pub coord: Coordinate,
    pub demand: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vehicle {
    pub id: usize,
    pub cap: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProblemBuilder {
    pub nodes: Vec<Node>,
    pub vehicle: Vehicle,
}

impl ProblemBuilder {
    pub fn new(nodes: Vec<Node>, vehicle: Vehicle) -> Self {
        Self { nodes, vehicle }
    }

    pub fn build(self) -> Problem {
        Problem {
            nodes: self.nodes,
            vehicle: self.vehicle,
        }
    }
}

#[derive(Debug)]
pub struct Problem {
    pub nodes: Vec<Node>,
    pub vehicle: Vehicle,
}

impl Problem {
    // Dimension of the problem
    pub fn dim(&self) -> usize {
        self.nodes.len()
    }

    // Number of customer nodes in the problem
    pub fn num_customers(&self) -> usize {
        self.dim() - 1
    }

    pub fn total_demand(&self) -> f64 {
        self.nodes.iter().map(|node| node.demand).sum()
    }

    pub fn max_demand(&self) -> Option<f64> {
        self.nodes
            .iter()
            .map(|node| node.demand)
            .max_by(|a, b| a.partial_cmp(&b).unwrap())
    }

    pub fn get_angle(&self, node: usize) -> i32 {
        let x = self.nodes[node].coord.lng - self.nodes[0].coord.lng;
        let y = self.nodes[node].coord.lat - self.nodes[0].coord.lat;
        let angle =
            (((y.atan2(x) / std::f64::consts::PI) * 32768.0).round() as i32).rem_euclid(65536);
        angle
    }

    pub fn from_mapping(&self, mapping: &[usize]) -> Self {
        Self {
            nodes: mapping
                .iter()
                .map(|&index| self.nodes[index].clone())
                .collect(),
            vehicle: self.vehicle.clone(),
        }
    }
}
