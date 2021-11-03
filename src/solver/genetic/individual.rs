use std::cmp::Ordering;

use crate::solver::Context;
use crate::solver::SolutionEvaluation;
use crate::utils::FloatCompare;

#[derive(Debug, Clone)]
pub struct Individual {
    // Used as key in population
    pub number: u64,
    // Solution representation as a giant tour. The tour does not include the depot
    pub genotype: Vec<usize>,
    // Solution representation split into routes
    pub phenotype: Vec<Vec<usize>>,
    // Biased fitness
    pub fitness: f64,
    // Evaluation of the solution
    pub evaluation: SolutionEvaluation,
}

impl Individual {
    pub fn new(genotype: Vec<usize>, number: u64) -> Self {
        Self {
            number,
            genotype,
            phenotype: Vec::new(),
            fitness: f64::INFINITY,
            evaluation: SolutionEvaluation::new(),
        }
    }

    pub fn new_random(ctx: &Context, number: u64) -> Self {
        let mut genotype: Vec<usize> = (1usize..ctx.problem.nodes.len()).collect();

        ctx.random.shuffle(genotype.as_mut_slice());

        let num_vehicles = ctx.config.borrow().num_vehicles as usize;
        Self {
            number,
            genotype,
            phenotype: vec![Vec::new(); num_vehicles], // Vec::with_capacity(num_vehicles),
            fitness: f64::INFINITY,
            evaluation: SolutionEvaluation::new(),
        }
    }

    // Returns the index of the node from the gene number in the genotype
    #[inline]
    pub fn genotype_node(&self, index: usize) -> usize {
        self.genotype[index - 1]
    }

    pub fn print(&self) {
        for (i, route) in self.phenotype.iter().enumerate() {
            print!("Route {}:", i + 1);
            for j in route {
                print!(" {}", j + 1);
            }
            println!("");
        }
    }

    pub fn num_nonempty_routes(&self) -> usize {
        self.phenotype
            .iter()
            .filter(|&route| route.len() > 0)
            .count()
    }

    pub fn num_routes(&self) -> usize {
        self.phenotype.len()
    }

    pub fn evaluate(&mut self, ctx: &Context) {
        self.evaluation.evaluate(ctx, &self.phenotype);
    }

    pub fn is_feasible(&self) -> bool {
        self.evaluation.is_feasible()
    }

    pub fn penalized_cost(&self) -> f64 {
        self.evaluation.penalized_cost
    }

    pub fn successor(&self, node: usize) -> usize {
        self.evaluation.successors[node]
    }

    pub fn predecessor(&self, node: usize) -> usize {
        self.evaluation.predecessors[node]
    }

    pub fn calculate_broken_pairs_distance(&self, other: &Self) -> i64 {
        let mut distance = 0;
        let size = self.genotype.len() + 1;

        for index in 1..size {
            // If the successor of self is neither the successor or predecessor of the other
            if self.successor(index) != other.successor(index)
                && self.successor(index) != other.predecessor(index)
            {
                distance += 1;
            }
            // If the predecessor of self is a depot, but neither the predecessor or successor of the other is a depot
            if self.predecessor(index) == 0
                && other.predecessor(index) != 0
                && other.successor(0) != 0
            {
                distance += 1;
            }
        }

        distance
    }

    pub fn sort_routes(&mut self, ctx: &Context) {
        let mut sorted_angles: Vec<(f64, usize)> = Vec::new();

        for (route_num, route) in self.phenotype.iter().enumerate() {
            if route.len() == 0 {
                sorted_angles.push((10.0, route_num));
                continue;
            }
            let mut x = 0.0;
            let mut y = 0.0;
            for &node in route.iter() {
                x += ctx.problem.nodes[node].coord.lng;
                y += ctx.problem.nodes[node].coord.lat;
            }
            x /= route.len() as f64;
            y /= route.len() as f64;

            x -= ctx.problem.nodes[0].coord.lng;
            y -= ctx.problem.nodes[0].coord.lat;
            let angle = y.atan2(x);
            sorted_angles.push((angle, route_num));
        }
        sorted_angles.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut new_phenotype = Vec::new();

        for &(_angle, route_num) in sorted_angles.iter() {
            new_phenotype.push(self.phenotype[route_num].clone());
        }
        self.phenotype = new_phenotype;

        // Update genotype
        let mut index = 0;
        for route in self.phenotype.iter() {
            for &node in route {
                self.genotype[index] = node;
                index += 1;
            }
        }
    }
}

impl PartialOrd for Individual {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.penalized_cost().partial_cmp(&other.penalized_cost())
        }
    }
}

impl PartialEq for Individual {
    fn eq(&self, other: &Self) -> bool {
        self.penalized_cost().approx_eq(other.penalized_cost())
    }
}

impl Ord for Individual {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("Failed to compare individuals")
    }
}

impl Eq for Individual {}
