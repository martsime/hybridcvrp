use crate::solver::Context;

/// Indices into the `Route` for a customer.
#[derive(Debug, Clone)]
pub struct NodeLocation {
    // Index into the vec of routes
    pub route_index: usize,
    // Index into the route
    pub node_index: usize,
}

impl NodeLocation {
    pub fn new(route_index: usize, node_index: usize) -> Self {
        Self {
            route_index,
            node_index,
        }
    }
    pub fn empty() -> Self {
        Self {
            route_index: 0,
            node_index: 0,
        }
    }

    pub fn update(&mut self, route_index: usize, node_index: usize) {
        self.route_index = route_index;
        self.node_index = node_index;
    }

    pub fn update_from_other(&mut self, other: &Self) {
        self.route_index = other.route_index;
        self.node_index = other.node_index;
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    // Nodes on the route in order
    pub nodes: Vec<usize>,

    // Distance of the route
    pub distance: f64,

    // Total overload on the route
    pub overload: f64,
}

impl Route {
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            distance: f64::default(),
            overload: f64::default(),
        }
    }
    pub fn remove(&mut self, index: usize, ctx: &Context) -> usize {
        let prev_node = if index == 0 { 0 } else { self.nodes[index - 1] };
        let next_node = if index == self.nodes.len() - 1 {
            0
        } else {
            self.nodes[index + 1]
        };

        // Update distance and overload
        self.distance += -ctx
            .matrix_provider
            .distance
            .get(prev_node, self.nodes[index])
            - ctx
                .matrix_provider
                .distance
                .get(self.nodes[index], next_node)
            + ctx.matrix_provider.distance.get(prev_node, next_node);

        self.overload -= ctx.problem.nodes[self.nodes[index]].demand;

        self.nodes.remove(index)
    }

    pub fn delta_distance(&self, index: usize, node: usize, ctx: &Context) -> f64 {
        let prev_node = if index == 0 { 0 } else { self.nodes[index - 1] };
        let next_node = if index == self.nodes.len() {
            0
        } else {
            self.nodes[index]
        };

        // Update distance and overload
        let delta = -ctx.matrix_provider.distance.get(prev_node, next_node)
            + ctx.matrix_provider.distance.get(prev_node, node)
            + ctx.matrix_provider.distance.get(node, next_node);
        delta
    }

    pub fn add(&mut self, index: usize, node: usize, ctx: &Context) {
        // Update distance and overload
        self.distance += self.delta_distance(index, node, ctx);

        self.overload += ctx.problem.nodes[node].demand;

        self.nodes.insert(index, node);
    }

    pub fn update_from_other(&mut self, other: &Self) {
        self.distance = other.distance;
        self.overload = other.overload;
        self.nodes.clear();
        for node in other.nodes.iter() {
            self.nodes.push(*node);
        }
    }
}
