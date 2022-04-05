use std::collections::VecDeque;

use crate::models::Matrix;
use crate::solver::genetic::Individual;
use crate::solver::Context;
use crate::utils::FloatCompare;

pub struct MyVecDeque<T> {
    queue: VecDeque<T>,
}

impl<T> MyVecDeque<T>
where
    T: Copy,
{
    pub fn new(size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(size),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn pop_back(&mut self) -> T {
        self.queue.pop_back().expect("No back")
    }

    pub fn pop_front(&mut self) -> T {
        self.queue.pop_front().expect("No front")
    }

    pub fn push_back(&mut self, value: T) {
        self.queue.push_back(value);
    }

    pub fn front(&self) -> &T {
        self.queue.front().expect("No front")
    }

    pub fn next_front(&mut self) -> T {
        let front = self.pop_front();
        let next_front = *self.front();
        self.push_front(front);
        next_front
    }

    pub fn back(&self) -> &T {
        self.queue.back().expect("No back")
    }

    pub fn push_front(&mut self, value: T) {
        self.queue.push_front(value);
    }
}

#[derive(Debug, Clone)]
pub struct NodeSplit {
    pub demand: f64,
    pub distance_depot: f64,
    pub distance_next: f64,
}

impl NodeSplit {
    pub fn new() -> Self {
        Self {
            demand: f64::default(),
            distance_depot: f64::default(),
            distance_next: f64::default(),
        }
    }
}

#[derive(Debug)]
pub struct Split {
    // Cost of shortest path from node 0 to index
    pub path_cost: Matrix<f64>,
    // List of predecessors
    pub predecessors: Matrix<usize>,

    // Information about nodes used in the split
    pub nodes: Vec<NodeSplit>,

    // Cumulative distance for each node
    pub cum_distance: Vec<f64>,
    // Cumulative load for each node
    pub cum_load: Vec<f64>,

    pub vehicle_cap: f64,
    pub penalty_capacity: f64,
}

impl Split {
    pub fn new(ctx: &Context) -> Self {
        let num_vehicles = ctx.config.borrow().num_vehicles as usize;
        Self {
            path_cost: Matrix::new(num_vehicles + 1, ctx.problem.dim()),
            predecessors: Matrix::new(num_vehicles + 1, ctx.problem.dim()),
            nodes: vec![NodeSplit::new(); ctx.problem.dim()],
            cum_distance: vec![f64::default(); ctx.problem.dim()],
            cum_load: vec![f64::default(); ctx.problem.dim()],
            vehicle_cap: ctx.problem.vehicle.cap,
            penalty_capacity: ctx.config.borrow().penalty_capacity,
        }
    }

    fn load(&mut self, ctx: &Context, individual: &Individual) {
        self.penalty_capacity = ctx.config.borrow().penalty_capacity;
        let num_nodes = ctx.problem.dim();
        for i in 1..num_nodes {
            let mut node = self.nodes.get_mut(i).expect("No node");
            let genotype_node = individual.genotype[i - 1];
            node.demand = ctx.problem.nodes[genotype_node].demand;
            node.distance_depot = ctx.matrix_provider.distance.get(genotype_node, 0) as f64;
            node.distance_next = if i < num_nodes - 1 {
                let next_genotype_node = individual.genotype[i];
                ctx.matrix_provider
                    .distance
                    .get(genotype_node, next_genotype_node) as f64
            } else {
                -1e30
            };
            self.cum_distance[i] = self.cum_distance[i - 1] + self.nodes[i - 1].distance_next;
            self.cum_load[i] = self.cum_load[i - 1] + self.nodes[i].demand;
        }
    }

    // Reset the path_cost
    fn reset(&mut self, limited_fleet: bool) {
        if limited_fleet {
            // Reset all
            self.path_cost.set(0, 0, 0.0);
            for row in 0..self.path_cost.rows {
                for col in 1..self.path_cost.cols {
                    self.path_cost.set(row, col, 1e30);
                }
            }
        } else {
            self.path_cost.set(0, 0, 0.0);
            for col in 1..self.path_cost.cols {
                self.path_cost.set(0, col, 1e30);
            }
        }
    }

    #[inline]
    fn propagate(&self, i: usize, j: usize, k: usize) -> f64 {
        self.path_cost.get(k, i) + self.cum_distance[j] - self.cum_distance[i + 1]
            + self.nodes[i + 1].distance_depot
            + self.nodes[j].distance_depot
            + self.penalty_capacity
                * (0f64.max(self.cum_load[j] - self.cum_load[i] - self.vehicle_cap) as f64)
    }

    #[inline]
    fn dominates(&self, i: usize, j: usize, k: usize) -> bool {
        self.path_cost.get(k, j) + self.nodes[j + 1].distance_depot
            > self.path_cost.get(k, i) + self.nodes[i + 1].distance_depot + self.cum_distance[j + 1]
                - self.cum_distance[i + 1]
                + self.penalty_capacity * ((self.cum_load[j] - self.cum_load[i]) as f64)
    }

    #[inline]
    fn dominates_right(&self, i: usize, j: usize, k: usize) -> bool {
        (self.path_cost.get(k, j) + self.nodes[j + 1].distance_depot).approx_lte(
            self.path_cost.get(k, i) + self.nodes[i + 1].distance_depot + self.cum_distance[j + 1]
                - self.cum_distance[i + 1],
        )
    }

    pub fn run(&mut self, ctx: &Context, individual: &mut Individual, max_vehicles: u64) {
        let max_vehicles = max_vehicles.max(ctx.vehicle_lower_bound()) as usize;
        self.load(ctx, individual);

        if !self.split(ctx, individual, max_vehicles) {
            self.split_limited_fleet(ctx, individual, max_vehicles);
        }
        individual.sort_routes(ctx);
        individual.evaluate(ctx);
    }

    // Split of the individual's genotype to create its phenotype
    pub fn split(
        &mut self,
        ctx: &Context,
        individual: &mut Individual,
        max_vehicles: usize,
    ) -> bool {
        self.reset(false);
        let dim = ctx.problem.dim();
        let cap = ctx.problem.vehicle.cap;
        let capacity_factor = ctx.config.borrow().split_capacity_factor;

        if ctx.config.borrow().linear_split {
            let mut queue: MyVecDeque<usize> = MyVecDeque::new(ctx.problem.dim());
            queue.push_back(0);

            for i in 1..dim {
                let front = *queue.front();
                self.path_cost.set(0, i, self.propagate(front, i, 0));
                self.predecessors.set(0, i, front);

                if i < dim - 1 {
                    if !self.dominates(*queue.back(), i, 0) {
                        while queue.len() > 0 && self.dominates_right(*queue.back(), i, 0) {
                            queue.pop_back();
                        }
                        queue.push_back(i);
                    }
                    while queue.len() > 1
                        && (self.propagate(*queue.front(), i + 1, 0)).approx_gte(self.propagate(
                            queue.next_front(),
                            i + 1,
                            0,
                        ))
                    {
                        queue.pop_front();
                    }
                }
            }
        } else {
            // Bellman-based split algorithm in O(nB) where B is the average route length
            for from_index in 0..(dim - 1) {
                let mut load = 0.0;
                let mut to_index = from_index + 1;
                let mut cost = 0.0;
                while to_index < dim
                    && (load + ctx.problem.nodes[individual.genotype_node(to_index)].demand)
                        .approx_lte(cap * capacity_factor)
                {
                    load += ctx.problem.nodes[individual.genotype_node(to_index)].demand;
                    if to_index == from_index + 1 {
                        cost = ctx
                            .matrix_provider
                            .distance
                            .get(0, individual.genotype_node(to_index));
                    } else {
                        cost += ctx.matrix_provider.distance.get(
                            individual.genotype_node(to_index - 1),
                            individual.genotype_node(to_index),
                        );
                    }
                    let mut new_path_cost = self.path_cost.get(0, from_index)
                        + cost
                        + ctx
                            .matrix_provider
                            .distance
                            .get(individual.genotype_node(to_index), 0);

                    if (load - cap).approx_gt(0.0) {
                        new_path_cost += (load - cap) * self.penalty_capacity;
                    }

                    if new_path_cost < self.path_cost.get(0, to_index) {
                        self.path_cost.set(0, to_index, new_path_cost);
                        self.predecessors.set(0, to_index, from_index);
                    }
                    to_index += 1;
                }
            }
        }

        individual.phenotype.clear();
        let mut end = ctx.problem.dim() - 1;
        while end > 0 {
            let mut new_route = Vec::new();
            let begin = self.predecessors.get(0, end);
            for index in begin..end {
                new_route.push(individual.genotype[index]);
            }
            individual.phenotype.push(new_route);
            end = begin;
        }

        let num_vehicles = individual.phenotype.len();
        let num_vehicles_ub = ctx.config.borrow().num_vehicles as usize;

        while individual.phenotype.len() < num_vehicles_ub {
            individual.phenotype.push(Vec::new());
        }

        // Return true if the split has fewer vehicles than the max allowed
        num_vehicles <= max_vehicles
    }

    pub fn split_limited_fleet(
        &mut self,
        ctx: &Context,
        individual: &mut Individual,
        max_vehicles: usize,
    ) -> bool {
        self.reset(true);
        let dim = ctx.problem.dim();
        let cap = ctx.problem.vehicle.cap;
        let capacity_factor = ctx.config.borrow().split_capacity_factor;

        if ctx.config.borrow().linear_split {
            let mut queue: MyVecDeque<usize> = MyVecDeque::new(ctx.problem.dim());

            for k in 0..max_vehicles {
                queue.clear();
                queue.push_back(k);

                for i in (k + 1)..dim {
                    if queue.is_empty() {
                        break;
                    }
                    self.path_cost
                        .set(k + 1, i, self.propagate(*queue.front(), i, k));
                    self.predecessors.set(k + 1, i, *queue.front());

                    if i < dim - 1 {
                        if !self.dominates(*queue.back(), i, k) {
                            while queue.len() > 0 && self.dominates_right(*queue.back(), i, k) {
                                queue.pop_back();
                            }
                            queue.push_back(i);
                        }
                        while queue.len() > 1
                            && (self.propagate(*queue.front(), i + 1, k))
                                .approx_gte(self.propagate(queue.next_front(), i + 1, k))
                        {
                            queue.pop_front();
                        }
                    }
                }
            }
        } else {
            for vehicle_index in 0..max_vehicles {
                for from_index in vehicle_index..(dim - 1) {
                    if self.path_cost.get(vehicle_index, from_index) > 1e29 {
                        break;
                    }
                    let mut load = 0.0;
                    let mut to_index = from_index + 1;
                    let mut cost = 0.0;
                    while to_index < dim
                        && load + ctx.problem.nodes[individual.genotype_node(to_index)].demand
                            <= cap * capacity_factor
                    {
                        load += ctx.problem.nodes[individual.genotype_node(to_index)].demand;
                        if to_index == from_index + 1 {
                            cost = ctx
                                .matrix_provider
                                .distance
                                .get(0, individual.genotype_node(to_index));
                        } else {
                            cost += ctx.matrix_provider.distance.get(
                                individual.genotype_node(to_index - 1),
                                individual.genotype_node(to_index),
                            );
                        }
                        let mut new_path_cost = self.path_cost.get(vehicle_index, from_index)
                            + cost
                            + ctx
                                .matrix_provider
                                .distance
                                .get(individual.genotype_node(to_index), 0);

                        if (load - cap).approx_gt(0.0) {
                            new_path_cost += (load - cap) * self.penalty_capacity;
                        }

                        if new_path_cost < self.path_cost.get(vehicle_index + 1, to_index) {
                            self.path_cost
                                .set(vehicle_index + 1, to_index, new_path_cost);
                            self.predecessors
                                .set(vehicle_index + 1, to_index, from_index);
                        }
                        to_index += 1;
                    }
                }
            }
        }

        // Find cheapest path with at most `max_vehicles` number of routes
        let last_customer_index = ctx.problem.dim() - 1;
        let mut min_cost = self.path_cost.get(max_vehicles, last_customer_index);
        let mut num_routes = max_vehicles;

        for vehicle_number in 1..max_vehicles {
            if self.path_cost.get(vehicle_number, last_customer_index) < min_cost {
                min_cost = self.path_cost.get(vehicle_number, last_customer_index);
                num_routes = vehicle_number;
            }
        }

        individual.phenotype.clear();
        let mut end = ctx.problem.dim() - 1;
        let mut vehicle_number = num_routes;
        while vehicle_number > 0 {
            let mut new_route = Vec::new();
            let begin = self.predecessors.get(vehicle_number, end);
            for index in begin..end {
                new_route.push(individual.genotype[index]);
            }
            individual.phenotype.insert(0, new_route);
            end = begin;
            vehicle_number -= 1;
        }

        let num_vehicles_ub = ctx.config.borrow().num_vehicles as usize;

        while individual.phenotype.len() < num_vehicles_ub {
            individual.phenotype.push(Vec::new());
        }

        // Return true if the split algorithm found a path from end to start
        end == 0
    }
}
