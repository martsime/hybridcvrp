use crate::models::{FloatType, IntType};
use crate::solver::improvement::RuinRecreateSolution;
use crate::solver::Context;

pub trait Ruin {
    fn run(&self, ctx: &Context, solution: &mut RuinRecreateSolution);
}

pub struct AdjacentStringRemoval {
    // Average number of customers ruined
    cavg: usize,
    // Maximum cardinality of the removed strings
    lmax: usize,
    // Split string factor
    alpha: FloatType,
}

impl AdjacentStringRemoval {
    pub fn new(ctx: &Context) -> Self {
        Self {
            cavg: ctx.config.borrow().average_ruin_cardinality,
            lmax: ctx.config.borrow().max_ruin_string_length,
            alpha: 0.01,
        }
    }
    fn average_tour_cardinality(&self, solution: &RuinRecreateSolution) -> FloatType {
        (solution
            .routes
            .iter()
            .map(|route| route.nodes.len() as FloatType)
            .sum::<FloatType>()
            / solution.routes.len() as FloatType)
            .round()
    }

    fn ruin_route(
        &self,
        ctx: &Context,
        solution: &mut RuinRecreateSolution,
        node: usize,
        route_number: usize,
        lt: usize,
    ) {
        let node_index = solution.locations[node].node_index;

        let route_length = solution.routes[route_number].nodes.len();

        // String procedure
        if ctx.random.real() < 0.5 {
            let min_start_index = (node_index as IntType - lt as IntType + 1).max(0) as usize;
            let max_start_index =
                (route_length as IntType - lt as IntType).min(node_index as IntType) as usize;

            let start_index = if min_start_index < max_start_index {
                ctx.random.range_usize(min_start_index, max_start_index + 1)
            } else {
                min_start_index
            };
            for _ in 0..lt {
                let removed = solution.routes[route_number].remove(start_index, ctx);
                solution.unassigned.push(removed);
            }

        // Split string procedure
        } else {
            let m_max = route_length - lt;
            let mut m = 1;

            if m_max > 0 {
                while m < m_max && ctx.random.real() > self.alpha {
                    m += 1;
                }
            } else {
                m = 0;
            }

            let remove_size = lt + m;

            let min_start_index =
                (node_index as IntType - remove_size as IntType + 1).max(0) as usize;
            let max_start_index = (route_length as IntType - remove_size as IntType)
                .min(node_index as IntType) as usize;

            let start_index = if min_start_index < max_start_index {
                ctx.random.range_usize(min_start_index, max_start_index + 1)
            } else {
                min_start_index
            };

            let m_index = ctx.random.range_usize(start_index, start_index + lt);

            let mut index = start_index + lt + m - 1;

            while index >= start_index {
                if index >= m_index + m || index < m_index {
                    let removed = solution.routes[route_number].remove(index, ctx);
                    solution.unassigned.push(removed);
                }

                if index == 0 {
                    break;
                } else {
                    index -= 1;
                }
            }
        }
        solution.ruined_routes.insert(route_number);
    }
}

impl Ruin for AdjacentStringRemoval {
    fn run(&self, ctx: &Context, solution: &mut RuinRecreateSolution) {
        // Equation 5
        let lsmax = self
            .average_tour_cardinality(solution)
            .min(self.lmax as FloatType);

        // Equation 6
        let ksmax = 4.0 * self.cavg as FloatType / (1.0 + lsmax) - 1.0;

        // Equation 7
        let ks = (ctx.random.real() * ksmax).floor() as usize + 1;

        // Initial customer
        let c_seed: usize = ctx.random.range_usize(1, ctx.problem.nodes.len());

        let neighbors = ctx.problem.neighbors(c_seed);

        for &neighbor in neighbors.iter() {
            let neighbor_route = solution.locations[neighbor].route_index;
            if solution.unassigned.contains(&neighbor)
                || solution.ruined_routes.contains(&neighbor_route)
            {
                continue;
            }

            let ltmax = lsmax.min(solution.routes[neighbor_route].nodes.len() as FloatType);

            let lt = (ctx.random.real() * ltmax).floor() as usize + 1;

            self.ruin_route(ctx, solution, neighbor, neighbor_route, lt);

            // Have ruined `ks` strings
            if solution.ruined_routes.len() >= ks {
                break;
            }
        }
    }
}
