use crate::solver::Context;

pub trait Metaheuristic {
    fn iterate(&mut self, ctx: &Context);
    fn terminated(&self) -> bool;
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
        while !self.metaheuristic.terminated() {
            self.metaheuristic.iterate(&self.ctx);
        }
        log::info!("Time: {:?}, Completed", self.ctx.elapsed());
    }
}
