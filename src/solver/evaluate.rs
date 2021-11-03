use crate::solver::Context;
use crate::utils::FloatCompare;

#[inline]
pub fn route_cost(distance: f64, overload: f64, penalty: f64) -> f64 {
    distance + penalty * 0f64.max(overload)
}

#[derive(Debug, Clone)]
pub struct RouteEvaluation {
    pub distance: f64,
    pub overload: f64,
    pub penalized_cost: f64,
}

impl RouteEvaluation {
    pub fn is_feasible(&self) -> bool {
        self.overload.approx_lte(0.0)
    }

    pub fn empty() -> Self {
        Self {
            distance: f64::MAX,
            overload: f64::MAX,
            penalized_cost: f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SolutionEvaluation {
    // Penalized cost of the solution
    pub penalized_cost: f64,
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
            penalized_cost: f64::INFINITY,
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
        let mut total_penalized_cost: f64 = 0.0;

        // If the solution is feasible
        let mut feasible = true;

        // Iterate over the routes in the solution
        for (route_index, route) in solution.iter().enumerate() {
            // Set the last_node to the depot and set the load to 0
            let mut last_node = depot_node;
            let mut load = 0.0;
            let mut route_distance = 0.0;

            // Iterate over the nodes on the route. Exclusive depot
            for &node in route.iter() {
                // Update distance
                route_distance += ctx.matrix_provider.distance.get(last_node, node);

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
            route_distance += ctx.matrix_provider.distance.get(last_node, depot_node);

            // Calculate the overload
            let overload = load - capacity;

            // Update distance and load for route
            self.routes[route_index].distance = route_distance;
            self.routes[route_index].overload = overload;

            // Add the penalized cost
            self.routes[route_index].penalized_cost =
                route_cost(route_distance, overload, penalty_capacity);
            total_penalized_cost += self.routes[route_index].penalized_cost;

            if overload.approx_gt(0.0) {
                // Update feasibility if the capacity is violated
                feasible = false;
            }
        }
        self.feasible = feasible;
        self.penalized_cost = total_penalized_cost;
    }
}
