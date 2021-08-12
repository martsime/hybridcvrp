use std::ptr;

use crate::solver::improvement::localsearch::Route;
use crate::solver::improvement::moves::ImprovementHeuristic;
use crate::solver::improvement::{insert_node, link_nodes, route_cost, LocalSearch, Node};
use crate::{constants::EPSILON, models::FloatType};

pub struct SwapOneWithOne;

impl ImprovementHeuristic for SwapOneWithOne {
    fn move_name(&self) -> &'static str {
        "SwapOneWithOne"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let u_prev = &*u.predecessor;
        let x = &*u.successor;

        let v = &*v_rc;
        let v_prev = &*v.predecessor;
        let y = &*v.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if u.number == y.number || u.number == v_prev.number {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_prev.number, u.number)
            - problem.distance.get(u.number, x.number)
            + problem.distance.get(u_prev.number, v.number)
            + problem.distance.get(v.number, x.number);

        let distance_two = r2.distance
            - problem.distance.get(v_prev.number, v.number)
            - problem.distance.get(v.number, y.number)
            + problem.distance.get(v_prev.number, u.number)
            + problem.distance.get(u.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            let v_demand = problem.nodes[v.number].demand;
            overload_one += -u_demand + v_demand;
            overload_two += u_demand - v_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("SwapOneWithOne");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let v_prev_rc = (*v_rc).predecessor;
        let y_rc = (*v_rc).successor;

        // Link (u_prev) -> (v)
        link_nodes(u_prev_rc, v_rc);

        // Link (v) -> (x)
        link_nodes(v_rc, x_rc);

        // Link (v_prev) -> (u)
        link_nodes(v_prev_rc, u_rc);

        // Link (u) -> (y)
        link_nodes(u_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct SwapTwoWithOne;

impl ImprovementHeuristic for SwapTwoWithOne {
    fn move_name(&self) -> &'static str {
        "SwapTwoWithOne"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let u_prev = &*u.predecessor;
        let x = &*u.successor;

        if x.is_depot() {
            return 0.0;
        }
        let x_next = &*x.successor;

        let v = &*v_rc;
        let v_prev = &*v.predecessor;
        let y = &*v.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if u.number == v_prev.number || x.number == v_prev.number || u.number == y.number {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_prev.number, u.number)
            - problem.distance.get(u.number, x.number)
            - problem.distance.get(x.number, x_next.number)
            + problem.distance.get(u_prev.number, v.number)
            + problem.distance.get(v.number, x_next.number);

        let distance_two = r2.distance
            - problem.distance.get(v_prev.number, v.number)
            - problem.distance.get(v.number, y.number)
            + problem.distance.get(v_prev.number, u.number)
            + problem.distance.get(u.number, x.number)
            + problem.distance.get(x.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            let v_demand = problem.nodes[v.number].demand;
            let x_demand = problem.nodes[x.number].demand;
            overload_one += -u_demand - x_demand + v_demand;
            overload_two += u_demand + x_demand - v_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("SwapTwoWithOne");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let v_prev_rc = (*v_rc).predecessor;
        let y_rc = (*v_rc).successor;

        // Link (u_prev) -> (v)
        link_nodes(u_prev_rc, v_rc);

        // Link (v) -> (x_next)
        link_nodes(v_rc, x_next_rc);

        // Link (v_prev) -> (u)
        link_nodes(v_prev_rc, u_rc);

        // Link (x) -> (y)
        link_nodes(x_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct SwapTwoWithTwo;

impl ImprovementHeuristic for SwapTwoWithTwo {
    fn move_name(&self) -> &'static str {
        "SwapTwoWithTwo"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;
        let u = &*u_rc;
        let u_prev = &*u.predecessor;
        let x = &*u.successor;

        if x.is_depot() {
            return 0.0;
        }
        let x_next = &*x.successor;

        let v = &*v_rc;
        let v_prev = &*v.predecessor;
        let y = &*v.successor;
        if y.is_depot() {
            return 0.0;
        }
        let y_next = &*y.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if u.number == y.number
            || v.number == x.number
            || y.number == u_prev.number
            || v.number == x_next.number
        {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_prev.number, u.number)
            - problem.distance.get(u.number, x.number)
            - problem.distance.get(x.number, x_next.number)
            + problem.distance.get(u_prev.number, v.number)
            + problem.distance.get(v.number, y.number)
            + problem.distance.get(y.number, x_next.number);

        let distance_two = r2.distance
            - problem.distance.get(v_prev.number, v.number)
            - problem.distance.get(v.number, y.number)
            - problem.distance.get(y.number, y_next.number)
            + problem.distance.get(v_prev.number, u.number)
            + problem.distance.get(u.number, x.number)
            + problem.distance.get(x.number, y_next.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            let v_demand = problem.nodes[v.number].demand;
            let x_demand = problem.nodes[x.number].demand;
            let y_demand = problem.nodes[y.number].demand;
            overload_one += -u_demand - x_demand + v_demand + y_demand;
            overload_two += u_demand + x_demand - v_demand - y_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("SwapTwoWithTwo");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let v_prev_rc = (*v_rc).predecessor;
        let y_rc = (*v_rc).successor;
        let y_next_rc = (*y_rc).successor;

        // Link (u_prev) -> (v)
        link_nodes(u_prev_rc, v_rc);

        // Link (y) -> (x_next)
        link_nodes(y_rc, x_next_rc);

        // Link (v_prev) -> (u)
        link_nodes(v_prev_rc, u_rc);

        // Link (x) -> (y_next)
        link_nodes(x_rc, y_next_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct BestSwapStar {
    pub cost: FloatType,
    pub u: *mut Node,
    pub v: *mut Node,
    pub pos_u: *mut Node,
    pub pos_v: *mut Node,
}

impl BestSwapStar {
    pub fn new() -> Self {
        Self {
            cost: FloatType::INFINITY,
            u: ptr::null_mut(),
            v: ptr::null_mut(),
            pos_u: ptr::null_mut(),
            pos_v: ptr::null_mut(),
        }
    }
}

pub struct SwapStar;

impl SwapStar {
    pub fn move_name() -> &'static str {
        "SwapStar"
    }
    pub unsafe fn run(ls: &mut LocalSearch, r1_ptr: *mut Route, r2_ptr: *mut Route) -> bool {
        let mut best_move = BestSwapStar::new();
        let problem = &ls.ctx.problem;
        ls.preprocess_insertions(r1_ptr, r2_ptr);
        ls.preprocess_insertions(r2_ptr, r1_ptr);

        // log::info!("Preprocessing done!");

        let r1 = &*r1_ptr;
        let r2 = &*r2_ptr;
        let mut u_ptr = (*r1.start_depot).successor;
        while !(*u_ptr).is_depot() {
            // log::info!("u_loop");
            let u = &*u_ptr;
            let mut v_ptr = (*r2.start_depot).successor;
            while !(*v_ptr).is_depot() {
                // log::info!("v_loop");
                let v = &*v_ptr;
                // TODO:
                let delta_penalty_r1 = 0.max(
                    r1.overload - problem.nodes[u.number].demand + problem.nodes[v.number].demand,
                ) as FloatType
                    * ls.penalty_capacity
                    - 0.max(r1.overload) as FloatType * ls.penalty_capacity;
                let delta_penalty_r2 = 0.max(
                    r2.overload + problem.nodes[u.number].demand - problem.nodes[v.number].demand,
                ) as FloatType
                    * ls.penalty_capacity
                    - 0.max(r2.overload) as FloatType * ls.penalty_capacity;

                if u.delta_removal as FloatType
                    + v.delta_removal as FloatType
                    + delta_penalty_r1
                    + delta_penalty_r2
                    <= 0.0
                {
                    let mut m = BestSwapStar::new();
                    m.u = u_ptr;
                    m.v = v_ptr;
                    let (best_pos_u, extra_v) = ls.cheapest_insert_and_removal(u_ptr, v_ptr);
                    let (best_pos_v, extra_u) = ls.cheapest_insert_and_removal(v_ptr, u_ptr);
                    m.pos_u = best_pos_u;
                    m.pos_v = best_pos_v;
                    m.cost = u.delta_removal as FloatType
                        + delta_penalty_r1
                        + extra_u
                        + v.delta_removal as FloatType
                        + delta_penalty_r2
                        + extra_v;
                    if m.cost < best_move.cost {
                        best_move = m;
                    }
                }

                v_ptr = v.successor;
            }
            u_ptr = u.successor;
        }

        let mut u_ptr = (*r1.start_depot).successor;
        while !(*u_ptr).is_depot() {
            let u = &*u_ptr;
            let mut m = BestSwapStar::new();
            m.u = u_ptr;
            let best_insert = &ls.best_inserts.get(r2.index, u.number).locations[0];
            m.pos_u = best_insert.node;
            let delta_penalty_r1 = 0.max(r1.overload - problem.nodes[u.number].demand) as FloatType
                * ls.penalty_capacity
                - 0.max(r1.overload) as FloatType * ls.penalty_capacity;
            let delta_penalty_r2 = 0.max(r2.overload + problem.nodes[u.number].demand) as FloatType
                * ls.penalty_capacity
                - 0.max(r2.overload) as FloatType * ls.penalty_capacity;
            m.cost = u.delta_removal as FloatType
                + best_insert.cost
                + delta_penalty_r1
                + delta_penalty_r2;

            if m.cost < best_move.cost {
                best_move = m;
            }

            u_ptr = u.successor;
        }

        let mut v_ptr = (*r2.start_depot).successor;
        while !(*v_ptr).is_depot() {
            let v = &*v_ptr;
            let mut m = BestSwapStar::new();
            m.v = v_ptr;
            let best_insert = &ls.best_inserts.get(r1.index, v.number).locations[0];
            m.pos_v = best_insert.node;
            let delta_penalty_r1 = 0.max(r1.overload + problem.nodes[v.number].demand) as FloatType
                * ls.penalty_capacity
                - 0.max(r1.overload) as FloatType * ls.penalty_capacity;
            let delta_penalty_r2 = 0.max(r2.overload - problem.nodes[v.number].demand) as FloatType
                * ls.penalty_capacity
                - 0.max(r2.overload) as FloatType * ls.penalty_capacity;
            m.cost = v.delta_removal as FloatType
                + best_insert.cost
                + delta_penalty_r1
                + delta_penalty_r2;

            if m.cost < best_move.cost {
                best_move = m;
            }

            v_ptr = v.successor;
        }

        if best_move.cost > -EPSILON {
            return false;
        }

        // ls.ctx
        //     .meta
        //     .add_improvement(Self::move_name(), -best_move.cost);

        ls.move_count += 1;
        if !best_move.pos_u.is_null() {
            insert_node(best_move.u, best_move.pos_u);
        }
        if !best_move.pos_v.is_null() {
            insert_node(best_move.v, best_move.pos_v);
        }

        // Update routes
        ls.update_route(r1_ptr);
        ls.update_route(r2_ptr);

        true
    }
}
