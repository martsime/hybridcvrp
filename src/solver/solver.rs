use crate::solver::Context;
use crate::utils;

pub trait Metaheuristic {
    fn iterate(&mut self, ctx: &Context);
    fn init(&mut self, ctx: &Context);
}

pub struct Solver<M>
where
    M: Metaheuristic,
{
    pub ctx: Context,
    pub metaheuristic: M,
}

impl<M> Solver<M>
where
    M: Metaheuristic,
{
    pub fn new(ctx: Context, metaheuristic: M) -> Self {
        Self { ctx, metaheuristic }
    }

    pub fn run(&mut self) {
        self.metaheuristic.init(&self.ctx);
        while !self.ctx.terminate() {
            self.metaheuristic.iterate(&self.ctx);
        }
        println!("Time: {:?}, Completed", self.ctx.elapsed());
        utils::write_solution_file(&self.ctx);
    }
}
