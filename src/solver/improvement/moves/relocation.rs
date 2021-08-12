use crate::models::FloatType;
use crate::solver::improvement::moves::ImprovementHeuristic;
use crate::solver::improvement::{link_nodes, route_cost, LocalSearch, Node};

pub struct RelocateSingle;

impl ImprovementHeuristic for RelocateSingle {
    fn move_name(&self) -> &'static str {
        "RelocateSingle"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let u_pred = &*u.predecessor;
        let x = &*u.successor;
        let v = &*v_rc;
        let y = &*v.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if y.number == u.number {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_pred.number, u.number)
            - problem.distance.get(u.number, x.number)
            + problem.distance.get(u_pred.number, x.number);

        let distance_two = r2.distance - problem.distance.get(v.number, y.number)
            + problem.distance.get(v.number, u.number)
            + problem.distance.get(u.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            overload_one += -u_demand;
            overload_two += u_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("RelocateSingle");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_pred_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let y_rc = (*v_rc).successor;

        // Update (up, u, x) -> (up, x)
        link_nodes(u_pred_rc, x_rc);

        // Update (v, y) -> (v, u, y)
        link_nodes(v_rc, u_rc);
        link_nodes(u_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct RelocateDouble;

impl ImprovementHeuristic for RelocateDouble {
    fn move_name(&self) -> &'static str {
        "RelocateDouble"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;
        let u = &*u_rc;
        let u_pred = &*u.predecessor;
        let x = &*u.successor;

        // Return if x is a depot.
        if x.is_depot() {
            return 0.0;
        }
        let x_next = &*x.successor;

        let v = &*v_rc;
        let y = &*v.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if u.number == y.number || v.number == x.number {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_pred.number, u.number)
            - problem.distance.get(u.number, x.number)
            - problem.distance.get(x.number, x_next.number)
            + problem.distance.get(u_pred.number, x_next.number);

        let distance_two = r2.distance - problem.distance.get(v.number, y.number)
            + problem.distance.get(v.number, u.number)
            + problem.distance.get(u.number, x.number)
            + problem.distance.get(x.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            let x_demand = problem.nodes[x.number].demand;
            overload_one += -u_demand - x_demand;
            overload_two += u_demand + x_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("RelocateDouble");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let y_rc = (*v_rc).successor;

        // Update (up, u, x, xn) -> (up, xn)
        link_nodes(u_prev_rc, x_next_rc);

        // Update (v, y) -> (v, u, x, y)
        link_nodes(v_rc, u_rc);
        link_nodes(x_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct RelocateDoubleReverse;

impl ImprovementHeuristic for RelocateDoubleReverse {
    fn move_name(&self) -> &'static str {
        "RelocateDoubleReverse"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut Node, v_rc: *mut Node) -> FloatType {
        let problem = &ls.ctx.problem;
        let u = &*u_rc;
        let u_prev = &*u.predecessor;
        let x = &*u.successor;

        // Return if x is a depot.
        if x.is_depot() {
            return 0.0;
        }
        let x_next = &*x.successor;

        let v = &*v_rc;
        let y = &*v.successor;

        let r1 = &*u.route;
        let r2 = &*v.route;

        // Nothing happens
        if u.number == y.number || v.number == x.number {
            return 0.0;
        }

        let distance_one = r1.distance
            - problem.distance.get(u_prev.number, u.number)
            - problem.distance.get(u.number, x.number)
            - problem.distance.get(x.number, x_next.number)
            + problem.distance.get(u_prev.number, x_next.number);

        let distance_two = r2.distance - problem.distance.get(v.number, y.number)
            + problem.distance.get(v.number, x.number)
            + problem.distance.get(x.number, u.number)
            + problem.distance.get(u.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = problem.nodes[u.number].demand;
            let x_demand = problem.nodes[x.number].demand;
            overload_one += -u_demand - x_demand;
            overload_two += u_demand + x_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut Node, v_rc: *mut Node) {
        log::debug!("RelocateDoubleReverse");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let y_rc = (*v_rc).successor;

        // Link (up) -> (xn)
        link_nodes(u_prev_rc, x_next_rc);

        // Link (v) -> (x)
        link_nodes(v_rc, x_rc);

        // Link (x) -> (u)
        link_nodes(x_rc, u_rc);

        // Link (u) -> (y)
        link_nodes(u_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}
