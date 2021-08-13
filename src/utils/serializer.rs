use std::fs::OpenOptions;
use std::io::Write;

use crate::solver::Context;

pub fn write_solution_file(ctx: &Context) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ctx.config.borrow().solution_path)
        .unwrap();

    if let Some(last_entry) = ctx.search_history.borrow().last_entry() {
        let mut route_number = 1;
        for route in last_entry.solution.routes.iter() {
            if route.len() > 0 {
                let mut route_string = format!("Route #{}:", route_number);
                for stop in route.iter() {
                    route_string.push_str(&format!(" {}", stop));
                }
                route_number += 1;
                write!(file, "{}\n", route_string).expect("Failed to write to solution file");
            }
        }
        write!(file, "Cost {}\n", last_entry.solution.cost.round() as u64)
            .expect("Failed to write to solution file");
    }
}
