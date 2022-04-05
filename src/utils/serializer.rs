use std::fs::OpenOptions;
use std::io::Write;

use crate::solver::Context;

pub fn write_solution_file(ctx: &Context) {
    if let Some(solution_path) = ctx.config.borrow().solution_path.as_ref() {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(solution_path)
            .unwrap();

        if let Some(last_entry) = ctx.search_history.borrow().last_entry() {
            writeln!(file, "{}", last_entry.solution).expect("Failed to write solution to file!");
        }
    }
}
