use crate::models::{FloatType, IntType};
use crate::solver::improvement::RuinRecreateSolution;
use crate::solver::Context;

fn sort_on_demand(ctx: &Context, solution: &mut RuinRecreateSolution) {
    let problem = &ctx.problem;
    solution
        .unassigned
        .sort_by(|&a, &b| problem.nodes[b].demand.cmp(&problem.nodes[a].demand))
}

fn sort_farthest_away_from_depot(ctx: &Context, solution: &mut RuinRecreateSolution) {
    let problem = &ctx.problem;
    solution
        .unassigned
        .sort_by(|&a, &b| problem.distance.get(b, 0).cmp(&problem.distance.get(a, 0)))
}

fn sort_closest_to_depot(ctx: &Context, solution: &mut RuinRecreateSolution) {
    let problem = &ctx.problem;
    solution
        .unassigned
        .sort_by(|&a, &b| problem.distance.get(a, 0).cmp(&problem.distance.get(b, 0)))
}

pub trait Recreate {
    fn run(&self, ctx: &Context, solution: &mut RuinRecreateSolution);
}

pub struct GreedyBlink {
    pub beta: f64,
}

impl GreedyBlink {
    pub fn sort_unassigned(&self, ctx: &Context, solution: &mut RuinRecreateSolution) {
        let number = ctx.random.range_usize(0, 11);
        if number < 4 {
            ctx.random.shuffle(solution.unassigned.as_mut_slice());
        } else if number < 8 {
            sort_on_demand(ctx, solution);
        } else if number < 10 {
            sort_farthest_away_from_depot(ctx, solution);
        } else if number < 11 {
            sort_closest_to_depot(ctx, solution);
        } else {
            unreachable!()
        }
    }
}

impl Default for GreedyBlink {
    fn default() -> Self {
        Self { beta: 0.01 }
    }
}

impl Recreate for GreedyBlink {
    fn run(&self, ctx: &Context, solution: &mut RuinRecreateSolution) {
        self.sort_unassigned(ctx, solution);
        let problem = &ctx.problem;

        let mut updated_routes = solution.ruined_routes.clone();
        while !solution.unassigned.is_empty() {
            let customer = solution.unassigned.remove(0);
            let demand = problem.nodes[customer].demand;

            let mut best_route: Option<usize> = None;
            let mut best_distance = IntType::MAX;
            let mut best_node_index = 0;

            for (route_number, route) in solution.routes.iter_mut().enumerate() {
                if route.overload + demand <= 0 {
                    // && // solution.ruined_routes.contains(&route_number) {
                    for index in 0..=route.nodes.len() {
                        let delta_distance = route.delta_distance(index, customer, ctx);
                        if delta_distance < best_distance {
                            best_distance = delta_distance;
                            best_node_index = index;
                            if let Some(best_route_number) = best_route.as_mut() {
                                *best_route_number = route_number;
                            } else {
                                best_route = Some(route_number);
                            }
                        }
                    }
                }
            }

            if let Some(best_route_number) = best_route {
                solution.routes[best_route_number].add(best_node_index, customer, ctx);
                updated_routes.insert(best_route_number);
            } else {
                log::info!("Cannot insert feasibly!");
                // Greedy insert infeasible
                let mut best_route: Option<usize> = None;
                let mut best_cost = FloatType::MAX;
                let mut best_node_index = 0;

                for (route_number, route) in solution.routes.iter_mut().enumerate() {
                    let overload = route.overload + demand;
                    let overload_cost =
                        0.max(overload) as FloatType * ctx.config.borrow().penalty_capacity;
                    for index in 0..=route.nodes.len() {
                        let delta_distance = route.delta_distance(index, customer, ctx);
                        let delta_cost = delta_distance as FloatType + overload_cost;
                        if delta_cost < best_cost {
                            best_cost = delta_cost;
                            best_node_index = index;
                            if let Some(best_route_number) = best_route.as_mut() {
                                *best_route_number = route_number;
                            } else {
                                best_route = Some(route_number);
                            }
                        }
                    }
                }
                let best_route_number = best_route.expect("No best route found");
                solution.routes[best_route_number].add(best_node_index, customer, ctx);
                updated_routes.insert(best_route_number);
            }
        }
        solution.evaluate(ctx, updated_routes.iter());
        solution.ruined_routes.clear();
    }
}
