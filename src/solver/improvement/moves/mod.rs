mod relocation;
pub use self::relocation::*;

mod swap;
pub use self::swap::*;

mod two_opt;
pub use self::two_opt::*;

use crate::models::FloatType;
use crate::solver::improvement::{LocalSearch, Node};
use crate::solver::Context;

pub trait ImprovementHeuristic {
    fn move_name(&self) -> &'static str;
    unsafe fn delta(&self, ls: &LocalSearch, node_u: *mut Node, node_v: *mut Node) -> FloatType;
    unsafe fn perform(&self, ls: &mut LocalSearch, node_u: *mut Node, node_v: *mut Node);
}

pub struct Moves {
    pub neighbor: Vec<Box<dyn ImprovementHeuristic>>,
    pub depot: Vec<Box<dyn ImprovementHeuristic>>,
    pub empty_route: Vec<Box<dyn ImprovementHeuristic>>,
}

impl Moves {
    pub fn new(ctx: &Context) -> Self {
        Self {
            neighbor: Self::neighborhood_moves(ctx),
            depot: Self::depot_moves(ctx),
            empty_route: Self::empty_route_moves(ctx),
        }
    }

    fn neighborhood_moves(ctx: &Context) -> Vec<Box<dyn ImprovementHeuristic>> {
        let mut moves: Vec<Box<dyn ImprovementHeuristic>> = Vec::new();
        if ctx.config.borrow().relocate_single {
            moves.push(Box::new(RelocateSingle));
        }
        if ctx.config.borrow().relocate_double {
            moves.push(Box::new(RelocateDouble));
        }
        if ctx.config.borrow().relocate_double_reverse {
            moves.push(Box::new(RelocateDoubleReverse));
        }
        if ctx.config.borrow().swap_one_with_one {
            moves.push(Box::new(SwapOneWithOne));
        }
        if ctx.config.borrow().swap_two_with_one {
            moves.push(Box::new(SwapTwoWithOne));
        }
        if ctx.config.borrow().swap_two_with_two {
            moves.push(Box::new(SwapTwoWithTwo));
        }
        if ctx.config.borrow().two_opt_intra_reverse {
            moves.push(Box::new(TwoOptIntraReverse));
        }
        if ctx.config.borrow().two_opt_inter_reverse {
            moves.push(Box::new(TwoOptInterReverse));
        }
        if ctx.config.borrow().two_opt_inter {
            moves.push(Box::new(TwoOptInter));
        }
        moves
    }

    fn depot_moves(ctx: &Context) -> Vec<Box<dyn ImprovementHeuristic>> {
        let mut moves: Vec<Box<dyn ImprovementHeuristic>> = Vec::new();
        if ctx.config.borrow().relocate_single {
            moves.push(Box::new(RelocateSingle));
        }
        if ctx.config.borrow().relocate_double {
            moves.push(Box::new(RelocateDouble));
        }
        if ctx.config.borrow().relocate_double_reverse {
            moves.push(Box::new(RelocateDoubleReverse));
        }
        if ctx.config.borrow().two_opt_inter_reverse {
            moves.push(Box::new(TwoOptInterReverse));
        }
        if ctx.config.borrow().two_opt_inter {
            moves.push(Box::new(TwoOptInter));
        }
        moves
    }

    fn empty_route_moves(ctx: &Context) -> Vec<Box<dyn ImprovementHeuristic>> {
        let mut moves: Vec<Box<dyn ImprovementHeuristic>> = Vec::new();
        if ctx.config.borrow().relocate_single {
            moves.push(Box::new(RelocateSingle));
        }
        if ctx.config.borrow().relocate_double {
            moves.push(Box::new(RelocateDouble));
        }
        if ctx.config.borrow().relocate_double_reverse {
            moves.push(Box::new(RelocateDoubleReverse));
        }
        if ctx.config.borrow().two_opt_inter {
            moves.push(Box::new(TwoOptInter));
        }
        moves
    }
}
