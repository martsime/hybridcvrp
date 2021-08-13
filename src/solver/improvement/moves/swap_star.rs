use std::ptr;

use crate::solver::improvement::localsearch::Route;
use crate::solver::improvement::{insert_node, LocalSearch, Node};
use crate::{constants::EPSILON, models::FloatType};

pub struct BestSwapStar {
    pub cost: FloatType,
    pub u: *mut Node,
    pub v: *mut Node,
    // Best position to insert `u` is right after `pos_u`
    pub pos_u: *mut Node,
    // Best position to insert `v` is right after `pos_v`
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
        // Setup local variables
        let mut best_move = BestSwapStar::new();
        let problem = &ls.ctx.problem;
        let r1 = &*r1_ptr;
        let r2 = &*r2_ptr;
        let mut u_ptr = (*r1.start_depot).successor;

        // Preprocess the three best insertions for all the nodes in the two routes
        ls.preprocess_insertions(r1_ptr, r2_ptr);
        ls.preprocess_insertions(r2_ptr, r1_ptr);

        // Loop over pairs of nodes in the two routes
        while !(*u_ptr).is_depot() {
            let u = &*u_ptr;
            let mut v_ptr = (*r2.start_depot).successor;
            while !(*v_ptr).is_depot() {
                let v = &*v_ptr;

                // Calculate the change in penalty when u and v swap routes
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

                // Filter to avoid moves with huge penalties due to violation of capacity constraints
                if u.delta_removal as FloatType
                    + v.delta_removal as FloatType
                    + delta_penalty_r1
                    + delta_penalty_r2
                    <= 0.0
                {
                    let mut m = BestSwapStar::new();
                    m.u = u_ptr;
                    m.v = v_ptr;

                    let (best_pos_u, delta_insertion_u) =
                        ls.cheapest_insert_and_removal(u_ptr, v_ptr);
                    let (best_pos_v, delta_insertion_v) =
                        ls.cheapest_insert_and_removal(v_ptr, u_ptr);
                    m.pos_u = best_pos_u;
                    m.pos_v = best_pos_v;
                    // Calculate change in cost for performing the move
                    m.cost = u.delta_removal as FloatType
                        + delta_penalty_r1
                        + delta_insertion_u
                        + v.delta_removal as FloatType
                        + delta_penalty_r2
                        + delta_insertion_v;

                    // Update the best move
                    if m.cost < best_move.cost {
                        best_move = m;
                    }
                }

                v_ptr = v.successor;
            }
            u_ptr = u.successor;
        }

        // Include all relocations of u into route of v.
        // This is very cheap as we already have calculated the best insertion positions
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

        // Include all relocations of v into route of u.
        // This is very cheap as we already have calculated the best insertion positions
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

        // Return false if the move does not reduce the objective function
        if best_move.cost > -EPSILON {
            return false;
        }

        ls.move_count += 1;

        // Relocate u into route of v
        if !best_move.pos_u.is_null() {
            insert_node(best_move.u, best_move.pos_u);
        }
        // Relocate u into route of v
        if !best_move.pos_v.is_null() {
            insert_node(best_move.v, best_move.pos_v);
        }

        // Update routes
        ls.update_route(r1_ptr);
        ls.update_route(r2_ptr);

        true
    }
}
