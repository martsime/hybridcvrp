use crate::solver::evaluate::route_cost;
use crate::solver::improvement::{LinkNode, LocalSearch, Move};

pub struct RelocateSingle;

impl Move for RelocateSingle {
    fn move_name(&self) -> &'static str {
        "RelocateSingle"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) -> f64 {
        let distance_matrix = &ls.ctx.matrix_provider.distance;
        let nodes = &ls.ctx.problem.nodes;

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
            - distance_matrix.get(u_pred.number, u.number)
            - distance_matrix.get(u.number, x.number)
            + distance_matrix.get(u_pred.number, x.number);

        let distance_two = r2.distance - distance_matrix.get(v.number, y.number)
            + distance_matrix.get(v.number, u.number)
            + distance_matrix.get(u.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = nodes[u.number].demand;
            overload_one += -u_demand;
            overload_two += u_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) {
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_pred_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let y_rc = (*v_rc).successor;

        // Update (up, u, x) -> (up, x)
        LinkNode::link_nodes(u_pred_rc, x_rc);

        // Update (v, y) -> (v, u, y)
        LinkNode::link_nodes(v_rc, u_rc);
        LinkNode::link_nodes(u_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct RelocateDouble;

impl Move for RelocateDouble {
    fn move_name(&self) -> &'static str {
        "RelocateDouble"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) -> f64 {
        let distance_matrix = &ls.ctx.matrix_provider.distance;
        let nodes = &ls.ctx.problem.nodes;

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
            - distance_matrix.get(u_pred.number, u.number)
            - distance_matrix.get(u.number, x.number)
            - distance_matrix.get(x.number, x_next.number)
            + distance_matrix.get(u_pred.number, x_next.number);

        let distance_two = r2.distance - distance_matrix.get(v.number, y.number)
            + distance_matrix.get(v.number, u.number)
            + distance_matrix.get(u.number, x.number)
            + distance_matrix.get(x.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = nodes[u.number].demand;
            let x_demand = nodes[x.number].demand;
            overload_one += -u_demand - x_demand;
            overload_two += u_demand + x_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) {
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let y_rc = (*v_rc).successor;

        // Update (up, u, x, xn) -> (up, xn)
        LinkNode::link_nodes(u_prev_rc, x_next_rc);

        // Update (v, y) -> (v, u, x, y)
        LinkNode::link_nodes(v_rc, u_rc);
        LinkNode::link_nodes(x_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}

pub struct RelocateDoubleReverse;

impl Move for RelocateDoubleReverse {
    fn move_name(&self) -> &'static str {
        "RelocateDoubleReverse"
    }
    unsafe fn delta(&self, ls: &LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) -> f64 {
        let distance_matrix = &ls.ctx.matrix_provider.distance;
        let nodes = &ls.ctx.problem.nodes;

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
            - distance_matrix.get(u_prev.number, u.number)
            - distance_matrix.get(u.number, x.number)
            - distance_matrix.get(x.number, x_next.number)
            + distance_matrix.get(u_prev.number, x_next.number);

        let distance_two = r2.distance - distance_matrix.get(v.number, y.number)
            + distance_matrix.get(v.number, x.number)
            + distance_matrix.get(x.number, u.number)
            + distance_matrix.get(u.number, y.number);

        let mut overload_one = r1.overload;
        let mut overload_two = r2.overload;

        if r1.index != r2.index {
            let u_demand = nodes[u.number].demand;
            let x_demand = nodes[x.number].demand;
            overload_one += -u_demand - x_demand;
            overload_two += u_demand + x_demand;
        }

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) {
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;

        let u_prev_rc = (*u_rc).predecessor;
        let x_rc = (*u_rc).successor;
        let x_next_rc = (*x_rc).successor;
        let y_rc = (*v_rc).successor;

        // Link (up) -> (xn)
        LinkNode::link_nodes(u_prev_rc, x_next_rc);

        // Link (v) -> (x)
        LinkNode::link_nodes(v_rc, x_rc);

        // Link (x) -> (u)
        LinkNode::link_nodes(x_rc, u_rc);

        // Link (u) -> (y)
        LinkNode::link_nodes(u_rc, y_rc);

        // Update routes
        ls.update_route(r1);
        if (*r1).index != (*r2).index {
            ls.update_route(r2);
        }
    }
}
