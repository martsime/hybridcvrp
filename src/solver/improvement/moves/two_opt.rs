use crate::models::FloatType;
use crate::solver::improvement::linked_list::{
    backward_reverse, forward_reverse, link_nodes, replace_end_depot, LinkNode,
};
use crate::solver::improvement::moves::Move;
use crate::solver::improvement::{route_cost, LocalSearch};

pub struct TwoOptIntraReverse;

impl Move for TwoOptIntraReverse {
    fn move_name(&self) -> &'static str {
        "TwoOptIntraReverse"
    }
    unsafe fn delta(
        &self,
        ls: &LocalSearch,
        u_rc: *mut LinkNode,
        v_rc: *mut LinkNode,
    ) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let v = &*v_rc;
        let r1 = &*u.route;
        let r2 = &*v.route;

        if r1.index != r2.index {
            return 0.0;
        }

        let x = &*u.successor;
        let y = &*v.successor;

        // Nothing happens
        if u.position > v.position || x.number == v.number {
            return 0.0;
        }

        let delta_distance = -problem.distance.get(u.number, x.number)
            - problem.distance.get(v.number, y.number)
            + problem.distance.get(u.number, v.number)
            + problem.distance.get(x.number, y.number);

        // Return delta cost
        delta_distance as FloatType
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) {
        log::debug!("TwoOptIntraReverse");
        let r1 = (*u_rc).route;
        let x_rc = (*u_rc).successor;
        let y_rc = (*v_rc).successor;
        backward_reverse(v_rc, x_rc, std::ptr::null_mut());
        link_nodes(u_rc, v_rc);
        link_nodes(x_rc, y_rc);
        ls.update_route(r1);
    }
}

pub struct TwoOptInterReverse;

impl Move for TwoOptInterReverse {
    fn move_name(&self) -> &'static str {
        "TwoOptInterReverse"
    }
    unsafe fn delta(
        &self,
        ls: &LocalSearch,
        u_rc: *mut LinkNode,
        v_rc: *mut LinkNode,
    ) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let v = &*v_rc;
        let r1 = &*u.route;
        let r2 = &*v.route;

        if r1.index == r2.index {
            return 0.0;
        }

        let x = &*u.successor;
        let y = &*v.successor;

        let cap = problem.vehicle.cap;

        let distance_one =
            u.cum_distance + v.cum_distance + problem.distance.get(u.number, v.number);
        let distance_two = r1.distance - x.cum_distance + r2.distance - y.cum_distance
            + problem.distance.get(x.number, y.number);
        let overload_one = u.cum_load + v.cum_load - cap;
        let overload_two = r1.load - u.cum_load + r2.load - v.cum_load - cap;

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, mut v_rc: *mut LinkNode) {
        log::debug!("TwoOptInterReverse");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;
        let mut x_rc = (*u_rc).successor;
        let y_rc = (*v_rc).successor;
        log::debug!("u: {}, route: {}", (*u_rc).number, *r1);
        log::debug!("v: {}, route: {}", (*v_rc).number, *r2);
        if !(*v_rc).is_depot() {
            backward_reverse(v_rc, std::ptr::null_mut(), (*r1).end_depot);
        } else {
            v_rc = (*r1).end_depot;
        }
        link_nodes(u_rc, v_rc);
        if !(*x_rc).is_depot() {
            forward_reverse(x_rc, std::ptr::null_mut(), (*r2).start_depot);
        } else {
            x_rc = (*r2).start_depot;
        }
        link_nodes(x_rc, y_rc);

        ls.update_route(r1);
        ls.update_route(r2);
    }
}

pub struct TwoOptInter;

impl Move for TwoOptInter {
    fn move_name(&self) -> &'static str {
        "TwoOptInter"
    }
    unsafe fn delta(
        &self,
        ls: &LocalSearch,
        u_rc: *mut LinkNode,
        v_rc: *mut LinkNode,
    ) -> FloatType {
        let problem = &ls.ctx.problem;

        let u = &*u_rc;
        let v = &*v_rc;
        let r1 = &*u.route;
        let r2 = &*v.route;

        if r1.index == r2.index {
            return 0.0;
        }

        let x = &*u.successor;
        let y = &*v.successor;

        let cap = problem.vehicle.cap;

        let distance_one = u.cum_distance + r2.distance - y.cum_distance
            + problem.distance.get(u.number, y.number);
        let distance_two = v.cum_distance + r1.distance - x.cum_distance
            + problem.distance.get(v.number, x.number);
        let overload_one = u.cum_load + r2.load - v.cum_load - cap;
        let overload_two = v.cum_load + r1.load - u.cum_load - cap;

        let old_cost = r1.cost + r2.cost;
        let new_cost = route_cost(distance_one, overload_one, ls.penalty_capacity)
            + route_cost(distance_two, overload_two, ls.penalty_capacity);

        // Return delta cost
        new_cost - old_cost
    }

    unsafe fn perform(&self, ls: &mut LocalSearch, u_rc: *mut LinkNode, v_rc: *mut LinkNode) {
        log::debug!("TwoOptInter");
        let r1 = (*u_rc).route;
        let r2 = (*v_rc).route;
        let x_rc = (*u_rc).successor;
        let y_rc = (*v_rc).successor;
        link_nodes(u_rc, y_rc);
        link_nodes(v_rc, x_rc);
        replace_end_depot(v_rc, (*r2).end_depot);
        replace_end_depot(u_rc, (*r1).end_depot);
        ls.update_route(r1);
        ls.update_route(r2);
    }
}
