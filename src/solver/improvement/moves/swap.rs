use crate::models::FloatType;
use crate::solver::improvement::moves::Move;
use crate::solver::improvement::{link_nodes, route_cost, LocalSearch, Node};

pub struct SwapOneWithOne;

impl Move for SwapOneWithOne {
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

impl Move for SwapTwoWithOne {
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

impl Move for SwapTwoWithTwo {
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
