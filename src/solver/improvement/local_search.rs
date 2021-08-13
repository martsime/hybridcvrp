use std::cmp::max;
use std::collections::HashSet;
use std::ptr;

use ahash::RandomState;

use crate::constants::EPSILON;
use crate::models::{FloatType, IntType, Matrix};
use crate::solver::genetic::Individual;
use crate::solver::improvement::linked_list::{link_nodes, LinkNode, LinkRoute};
use crate::solver::improvement::moves::{Moves, SwapStar};
use crate::solver::Context;

#[inline]
pub fn route_cost(distance: IntType, overload: IntType, penalty: FloatType) -> FloatType {
    distance as FloatType + penalty * max(0, overload) as FloatType
}

#[derive(Debug, Clone, Copy)]
pub struct InsertLocation {
    pub cost: FloatType,
    pub node: *mut LinkNode,
}

impl InsertLocation {
    pub fn new() -> Self {
        Self {
            cost: FloatType::INFINITY,
            node: ptr::null_mut(),
        }
    }

    pub fn reset(&mut self) {
        self.cost = FloatType::INFINITY;
        self.node = ptr::null_mut();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThreeBestInserts {
    pub locations: [InsertLocation; 3],
    pub last_calculated: IntType,
}

impl ThreeBestInserts {
    pub fn new() -> Self {
        Self {
            locations: [InsertLocation::new(); 3],
            last_calculated: 0,
        }
    }

    pub fn reset(&mut self) {
        for loc in self.locations.iter_mut() {
            loc.reset();
        }
    }

    pub fn add(&mut self, loc: InsertLocation) {
        if loc.cost > self.locations[2].cost {
        } else if loc.cost > self.locations[1].cost {
            self.locations[2] = loc;
        } else if loc.cost > self.locations[0].cost {
            self.locations[2] = self.locations[1];
            self.locations[1] = loc;
        } else {
            self.locations[2] = self.locations[1];
            self.locations[1] = self.locations[0];
            self.locations[0] = loc;
        }
    }
}

pub struct LocalSearch {
    pub ctx: &'static Context,

    pub routes: Vec<LinkRoute>,
    pub customers: Vec<usize>,
    pub correlations: Vec<usize>,
    pub granularities: Vec<usize>,
    pub num_correlations: usize,

    pub move_count: IntType,
    pub moves: Moves,

    // Store the three best insert locations for each node and route. Used by swap start move.
    pub best_inserts: Matrix<ThreeBestInserts>,

    // Nodes used in the local search
    pub nodes: Vec<LinkNode>,
    pub start_depots: Vec<LinkNode>,
    pub end_depots: Vec<LinkNode>,

    // Indices to empty routes
    pub empty_routes: HashSet<usize, RandomState>,

    // Penalty for capacity during the search
    pub penalty_capacity: FloatType,
}

impl LocalSearch {
    pub fn new(ctx: &Context, penalty_multiplier: FloatType) -> Self {
        unsafe {
            // Create all the nodes
            let nodes: Vec<LinkNode> = ctx
                .problem
                .nodes
                .iter()
                .enumerate()
                .map(|(index, _)| LinkNode::new(index, ctx.problem.get_angle(index)))
                .collect();

            let customers: Vec<usize> = (1..ctx.problem.dim()).collect();

            // Create nodes for the depots
            let num_vehicles = ctx.config.borrow().num_vehicles as usize;

            // Create depots and routes
            let mut start_depots = Vec::with_capacity(num_vehicles);
            let mut end_depots = Vec::with_capacity(num_vehicles);
            let mut routes = Vec::with_capacity(num_vehicles);
            for route_number in 0..num_vehicles {
                let start_depot = LinkNode::new(0, 0);
                let end_depot = LinkNode::new(0, 0);
                start_depots.push(start_depot);
                end_depots.push(end_depot);
                let start_depot_ptr = start_depots.get_unchecked_mut(route_number) as *mut LinkNode;
                let end_depot_ptr = end_depots.get_unchecked_mut(route_number) as *mut LinkNode;
                routes.push(LinkRoute::new(route_number, start_depot_ptr, end_depot_ptr));
            }

            Self {
                ctx: &*(ctx as *const Context),
                moves: Moves::new(ctx),
                nodes,
                customers,
                best_inserts: Matrix::init(
                    ThreeBestInserts::new(),
                    num_vehicles,
                    ctx.problem.dim(),
                ),
                correlations: ctx.problem.correlations.clone(),
                granularities: ctx.problem.granularities.clone(),
                num_correlations: ctx.problem.num_correlations,
                routes: routes,
                move_count: 0,
                empty_routes: HashSet::with_capacity_and_hasher(
                    num_vehicles,
                    ctx.random.random_state(),
                ),
                start_depots: start_depots,
                end_depots: end_depots,
                penalty_capacity: ctx.config.borrow().penalty_capacity * penalty_multiplier,
            }
        }
    }

    pub fn update_penalty(&mut self, penalty_multiplier: FloatType) {
        self.penalty_capacity = self.ctx.config.borrow().penalty_capacity * penalty_multiplier;
    }

    pub fn load_individual(&mut self, individual: &Individual) {
        unsafe {
            for (route_index, route) in individual.phenotype.iter().enumerate() {
                // Start with the depot as the last node
                let mut last_node = &mut self.start_depots[route_index] as *mut LinkNode;

                // Link up all nodes
                for &node_index in route.iter() {
                    let node = &mut self.nodes[node_index] as *mut LinkNode;
                    link_nodes(last_node, node);
                    last_node = node;
                }

                // Link the last node to the end depot
                let depot_end = &mut self.end_depots[route_index] as *mut LinkNode;
                link_nodes(last_node, depot_end);

                let route = &mut self.routes[route_index] as *mut LinkRoute;
                (*route).last_tested_swap_star = -1;
                for node_number in 0..self.nodes.len() {
                    let best_insert = self.best_inserts.get_mut(route_index, node_number);
                    best_insert.reset();
                    best_insert.last_calculated = -1;
                }
                self.update_route(route);
            }
        }
    }

    pub fn reset(&mut self) {
        self.move_count = 0;
        for node in self.nodes.iter_mut() {
            node.last_tested = -1;
        }
    }

    pub fn run(
        &mut self,
        ctx: &Context,
        individual: &mut Individual,
        penalty_multiplier: FloatType,
    ) {
        unsafe {
            self.ctx = &*(ctx as *const Context);
            self.reset();
            self.update_penalty(penalty_multiplier);
            self.load_individual(individual);
            self.search();
        }
        self.update_individual(individual);
    }

    unsafe fn search(&mut self) {
        let mut loop_count = 0;
        let mut improvement = true;
        let moves = &*{ &self.moves as *const Moves };
        while improvement {
            improvement = false;
            // Loop over all customers in random order
            self.ctx.random.shuffle(self.customers.as_mut_slice());
            let customers = &*{ &self.customers as *const Vec<usize> };
            for u_index in customers {
                // Get all correlated customers in random order
                let start_index = self.num_correlations * *u_index;
                let end_index = start_index + self.granularities[*u_index];

                let cor = &mut *{
                    self.correlations.get_unchecked_mut(start_index..end_index) as *mut [usize]
                };

                if self.ctx.random.range_usize(0, cor.len()) == 0 {
                    self.ctx.random.shuffle(cor);
                }

                let u = &mut self.nodes[*u_index] as *mut LinkNode;
                let mut route_u = (*u).route;

                // Update RI timestamp for node u
                let last_test_u = (*u).last_tested;
                (*u).last_tested = self.move_count;

                // Iterate over correlated nodes
                'v_loop: for &v_index in cor.iter() {
                    let v = &mut self.nodes[v_index] as *mut LinkNode;
                    let route_v = (*v).route;

                    // Only try moves if one of the routes is modified since last time
                    if loop_count == 0
                        || max((*route_u).last_modified, (*route_v).last_modified) > last_test_u
                    {
                        for m in moves.neighbor.iter() {
                            let delta = m.delta(&self, u, v);
                            if delta + EPSILON < 0.0 {
                                self.move_count += 1;
                                m.perform(self, u, v);
                                route_u = (*u).route;
                                improvement = true;
                                // self.ctx.meta.add_improvement(m.move_name(), -delta);
                                continue 'v_loop;
                            }
                        }
                        let v_pred = (*v).predecessor;
                        if (*v_pred).is_depot() {
                            for m in moves.depot.iter() {
                                let delta = m.delta(&self, u, v);
                                if delta + EPSILON < 0.0 {
                                    self.move_count += 1;
                                    m.perform(self, u, v);
                                    route_u = (*u).route;
                                    improvement = true;
                                    // self.ctx.meta.add_improvement(m.move_name(), -delta);
                                    continue 'v_loop;
                                }
                            }
                        }
                    }
                }
                if loop_count > 0 && !self.empty_routes.is_empty() {
                    let empty_route_index =
                        *self.empty_routes.iter().next().expect("No empty route");
                    let route_v = &mut self.routes[empty_route_index] as *mut LinkRoute;
                    let v = (*route_v).start_depot;
                    for m in moves.empty_route.iter() {
                        let delta = m.delta(&self, u, v);
                        if delta + EPSILON < 0.0 {
                            self.move_count += 1;
                            m.perform(self, u, v);
                            improvement = true;
                            // self.ctx.meta.add_improvement(m.move_name(), -delta);
                            break;
                        }
                    }
                }
            }
            if self.ctx.config.borrow().swap_star {
                for r1_num in 0..self.routes.len() {
                    let r1_ptr = &mut self.routes[r1_num] as *mut LinkRoute;
                    let last_tested_u = (*r1_ptr).last_tested_swap_star;
                    (*r1_ptr).last_tested_swap_star = self.move_count;
                    for r2_num in (r1_num + 1)..self.routes.len() {
                        let r2_ptr = &mut self.routes[r2_num] as *mut LinkRoute;
                        if !(*r1_ptr).is_empty()
                            && !(*r2_ptr).is_empty()
                            && r1_num < r2_num
                            && (loop_count == 0 || {
                                (*r1_ptr)
                                    .last_tested_swap_star
                                    .max((*r2_ptr).last_tested_swap_star)
                                    > last_tested_u
                            })
                        {
                            if (*r1_ptr).sector.overlaps(&(*r2_ptr).sector) {
                                if SwapStar::run(self, r1_ptr, r2_ptr) {
                                    improvement = true;
                                }
                            }
                        }
                    }
                }
                loop_count += 1;
            }
        }
    }

    fn update_individual(&self, individual: &mut Individual) {
        // Clear the genotype
        individual.genotype.clear();

        unsafe {
            // Loop over the routes and update the genotype and the phenotype
            for (route_number, route) in self.routes.iter().enumerate() {
                let mut phenotype_nodes: Vec<usize> = Vec::with_capacity(route.num_customers);
                let mut next_node = route.start_depot;
                while !next_node.is_null() {
                    let node = &*next_node;
                    if !node.is_depot() {
                        phenotype_nodes.push(node.number);
                    }
                    next_node = node.successor;
                }
                individual.genotype.extend(phenotype_nodes.iter());
                individual.phenotype[route_number] = phenotype_nodes;
            }

            // Reevaluate the individual
            individual.sort_routes(self.ctx);
            individual.evaluate(self.ctx);
        }
    }

    // Used to update the route after a move is performed
    pub fn update_route(&mut self, route_ptr: *mut LinkRoute) {
        let problem = &self.ctx.problem;
        unsafe {
            // Variables to be calculated for the route
            let mut distance = 0;
            let mut load = 0;
            let mut num_customers = 0;

            // Start with the depot as the first node
            let mut prev_node_ptr = (*route_ptr).start_depot;

            // Update information for the start depot
            (*prev_node_ptr).route = route_ptr;
            (*prev_node_ptr).position = 0;

            // Reset the route circle sector
            (*route_ptr).sector.reset();

            // Go to the next node
            let mut node_ptr = (*prev_node_ptr).successor;
            let mut position = 1;

            // Loop through all nodes in route
            while !node_ptr.is_null() {
                // Add distance and load for the node
                distance += problem
                    .distance
                    .get((*prev_node_ptr).number, (*node_ptr).number);
                load += problem.nodes[(*node_ptr).number].demand;

                // Update circle sector for customers
                if !(*node_ptr).is_depot() {
                    (*route_ptr).sector.extend((*node_ptr).angle);
                    num_customers += 1;
                }

                // Update information on the node
                (*node_ptr).cum_distance = distance;
                (*node_ptr).cum_load = load;
                (*node_ptr).route = route_ptr;
                (*node_ptr).position = position;

                // Increment position and pointers
                position += 1;
                prev_node_ptr = node_ptr;
                node_ptr = (*node_ptr).successor;
            }

            // Update information on the route
            (*route_ptr).distance = distance;
            (*route_ptr).load = load;
            (*route_ptr).overload = load - problem.vehicle.cap;
            (*route_ptr).last_modified = self.move_count;
            (*route_ptr).num_customers = num_customers;

            // Ensure predecessor of start_depot and successor of end_depot are null
            self.start_depots[(*route_ptr).index].predecessor = ptr::null_mut();
            self.end_depots[(*route_ptr).index].successor = ptr::null_mut();

            // Update route cost
            (*route_ptr).cost = route_cost(
                (*route_ptr).distance,
                (*route_ptr).overload,
                self.penalty_capacity,
            );

            // Update set of empty routes
            if (*route_ptr).is_empty() {
                self.empty_routes.insert((*route_ptr).index);
            } else {
                self.empty_routes.remove(&(*route_ptr).index);
            }
        }
    }

    /// Used to preprocess the three best insertion costs for all nodes in a pair of routes
    pub unsafe fn preprocess_insertions(&mut self, r1_ptr: *mut LinkRoute, r2_ptr: *mut LinkRoute) {
        let problem = &self.ctx.problem;
        let r1 = &*r1_ptr;
        let r2 = &*r2_ptr;

        // Start with the first customer in route 1
        let mut u_ptr = (*r1.start_depot).successor;

        // Loop over all customers in route 1
        while !(*u_ptr).is_depot() {
            // Derefence pointers
            let u = &*u_ptr;
            let u_prev = &*u.predecessor;
            let x = &*u.successor;

            // Calculate and set change in objective when removing u
            let delta_removal = problem.distance.get(u_prev.number, x.number)
                - problem.distance.get(u_prev.number, u.number)
                - problem.distance.get(u.number, x.number);
            (*u_ptr).delta_removal = delta_removal;

            // Only recalculate insertion cost into route 2 if the route has changed since last calculation
            if r2.last_modified > self.best_inserts.get(r2.index, u.number).last_calculated {
                // Reset best inserts of u into route 2
                self.best_inserts.get_mut(r2.index, u.number).reset();
                self.best_inserts
                    .get_mut(r2.index, u.number)
                    .last_calculated = self.move_count;

                // Start with first customer in the second route as v
                let mut v_ptr = (*r2.start_depot).successor;

                // Check cost of inserting node u between the start depot and the first node in route 2
                let cost = problem.distance.get(0, u.number)
                    + problem.distance.get(u.number, (*v_ptr).number)
                    - problem.distance.get(0, (*v_ptr).number);
                self.best_inserts
                    .get_mut(r2.index, u.number)
                    .add(InsertLocation {
                        cost: cost as FloatType,
                        node: r2.start_depot,
                    });

                // Calculate insertion cost of u for the remaining positions in route 2
                while !(*v_ptr).is_depot() {
                    let v = &*v_ptr;
                    let y = &*v.successor;
                    let delta_insert = problem.distance.get(v.number, u.number)
                        + problem.distance.get(u.number, y.number)
                        - problem.distance.get(v.number, y.number);
                    let cost = delta_insert as FloatType;

                    self.best_inserts
                        .get_mut(r2.index, u.number)
                        .add(InsertLocation { cost, node: v_ptr });

                    v_ptr = v.successor;
                }
            }
            u_ptr = u.successor;
        }
    }

    /// Finds the cheapest insert location of u into the route of v,
    /// while v is removed at the same time
    pub unsafe fn cheapest_insert_and_removal(
        &mut self,
        u_ptr: *mut LinkNode,
        v_ptr: *mut LinkNode,
    ) -> (*mut LinkNode, FloatType) {
        // Derefence pointers and setup local variables
        let u = &*u_ptr;
        let v = &*v_ptr;
        let r2 = &(*v.route);
        let problem = &self.ctx.problem;

        // Start with the best insertion into route v.
        let best_insertion = self.best_inserts.get_mut(r2.index, u.number);
        let mut best_node = best_insertion.locations[0].node;
        let mut best_cost = best_insertion.locations[0].cost;

        // Found is true if the best insert position is neither directly before or after v.
        // If the best insert position involves v, the position is illegal when v is removed,
        // and thus we must use the second or third best insert position.
        let mut found =
            (*best_node).number != v.number && (*(*best_node).successor).number != v.number;
        if !found && !best_insertion.locations[1].node.is_null() {
            best_node = best_insertion.locations[1].node;
            best_cost = best_insertion.locations[1].cost;
            found = (*best_node).number != v.number && (*(*best_node).successor).number != v.number;
            if !found && !best_insertion.locations[2].node.is_null() {
                best_node = best_insertion.locations[2].node;
                best_cost = best_insertion.locations[2].cost;
                found = true;
            }
        }

        let v_prev = &*(v.predecessor);
        let y = &*(v.successor);

        // Calculate the cost of inserting u in place of v, as
        // the best position already found is into route 2 while v is still present
        let delta_cost = (problem.distance.get(v_prev.number, u.number)
            + problem.distance.get(u.number, y.number)
            - problem.distance.get(v_prev.number, y.number)) as FloatType;

        // Update best insertion if it's cheaper to insert in place of v
        if !found || delta_cost < best_cost {
            best_node = v.predecessor;
            best_cost = delta_cost;
        }

        // Returns the best insert position (right after the `best_node`) with a cost of `best_cost`
        (best_node, best_cost)
    }
}

impl Drop for LocalSearch {
    fn drop(&mut self) {
        for node in self.nodes.iter_mut() {
            node.route = ptr::null_mut();
            node.predecessor = ptr::null_mut();
            node.successor = ptr::null_mut();
        }
        for node in self.start_depots.iter_mut() {
            node.route = ptr::null_mut();
            node.predecessor = ptr::null_mut();
            node.successor = ptr::null_mut();
        }
        for node in self.end_depots.iter_mut() {
            node.route = ptr::null_mut();
            node.predecessor = ptr::null_mut();
            node.successor = ptr::null_mut();
        }
    }
}
