use std::cmp::max;

use crate::models::{FloatType, IntType};
use crate::solver::Context;

#[inline]
pub fn route_cost(distance: IntType, overload: IntType, penalty: FloatType) -> FloatType {
    distance as FloatType + penalty * max(0, overload) as FloatType
}

#[derive(Debug, Clone)]
pub struct RouteEvaluation {
    pub distance: IntType,
    pub overload: IntType,
    pub penalized_cost: FloatType,
}

impl RouteEvaluation {
    pub fn is_feasible(&self) -> bool {
        self.overload <= 0
    }

    pub fn empty() -> Self {
        Self {
            distance: IntType::MAX,
            overload: IntType::MAX,
            penalized_cost: FloatType::INFINITY,
        }
    }
    
}

#[derive(Debug, Clone)]
pub struct SolutionEvaluation {
    // Penalized cost of the solution
    pub penalized_cost: FloatType,
    pub feasible: bool,

    // Evaluation of routes
    pub routes: Vec<RouteEvaluation>,

    // For every node in the solution, keep track of the predecessor
    pub predecessors: Vec<usize>,
    // For every node in the solution, keep track of the successor
    pub successors: Vec<usize>,
}

impl SolutionEvaluation {
    pub fn new() -> Self {
        Self {
            penalized_cost: FloatType::INFINITY,
            feasible: false,
            routes: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn is_feasible(&self) -> bool {
        self.feasible
    }

    fn reset(&mut self, num_routes: usize, num_nodes: usize) {
        self.routes.resize(num_routes, RouteEvaluation::empty());
        self.predecessors.resize(num_nodes, 0);
        self.successors.resize(num_nodes, 0);
    }

    pub fn evaluate(&mut self, ctx: &Context, solution: &Vec<Vec<usize>>) {
        // Reset vecs
        self.reset(solution.len(), ctx.problem.dim());

        // Variables to make the algorithm more readable
        let depot_node = 0;
        let capacity = ctx.problem.vehicle.cap;
        let penalty_capacity = ctx.config.borrow().penalty_capacity;

        // Total cost of the solution
        let mut total_penalized_cost: FloatType = 0.0;

        // If the solution is feasible
        let mut feasible = true;

        // Iterate over the routes in the solution
        for (route_index, route) in solution.iter().enumerate() {
            // Set the last_node to the depot and set the load to 0
            let mut last_node = depot_node;
            let mut load = 0;
            let mut route_distance = 0;

            // Iterate over the nodes on the route. Exclusive depot
            for &node in route.iter() {
                // Update distance
                route_distance += ctx.problem.distance.get(last_node, node);

                // Update load on route
                load += ctx.problem.nodes[node].demand;

                // Update predecessors and successors of nodes
                self.predecessors[node] = last_node;
                self.successors[last_node] = node;

                // Update last_node to current node
                last_node = node;
            }
            // Set successor of last node in a route to the depot
            self.successors[last_node] = depot_node;

            // Add the distance from the last node in a route and to the depot
            route_distance += ctx.problem.distance.get(last_node, depot_node);

            // Calculate the overload
            let overload = load - capacity;

            // Update distance and load for route
            self.routes[route_index].distance = route_distance;
            self.routes[route_index].overload = overload;

            // Add the penalized cost
            self.routes[route_index].penalized_cost =
                route_cost(route_distance, overload, penalty_capacity);
            total_penalized_cost += self.routes[route_index].penalized_cost;

            if overload > 0 {
                // Update feasibility if the capacity is violated
                feasible = false;
            }
        }
        self.feasible = feasible;
        self.penalized_cost = total_penalized_cost;
    }
}
