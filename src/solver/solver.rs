use crate::solver::{Context, SearchHistory};

pub trait Metaheuristic {
    fn iterate(&mut self, ctx: &Context) -> bool;
    fn init(&mut self, ctx: &Context);
    fn history(&self) -> &SearchHistory;
    fn print(&self);
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

    pub fn start(&mut self) {
        self.metaheuristic.init(&self.ctx);
        while !self.metaheuristic.iterate(&self.ctx) {}
        self.metaheuristic.print();
    }
}
