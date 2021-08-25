use std::ptr;

use crate::models::FloatType;
use crate::models::IntType;
use crate::solver::improvement::LinkNode;

/// Used to store the cost of inserting directly after `node`
#[derive(Debug, Clone, Copy)]
pub struct InsertLocation {
    pub cost: FloatType,
    pub node: *mut LinkNode,
}

impl InsertLocation {
    pub fn new() -> Self {
        Self {
            cost: FloatType::INFINITY,
            node: ptr::null_mut(),
        }
    }

    pub fn reset(&mut self) {
        self.cost = FloatType::INFINITY;
        self.node = ptr::null_mut();
    }
}

/// Stores the three best insertion locations for a node
#[derive(Debug, Clone, Copy)]
pub struct ThreeBestInserts {
    pub locations: [InsertLocation; 3],
    pub last_calculated: IntType,
}

impl ThreeBestInserts {
    pub fn new() -> Self {
        Self {
            locations: [InsertLocation::new(); 3],
            last_calculated: 0,
        }
    }

    pub fn reset(&mut self) {
        for loc in self.locations.iter_mut() {
            loc.reset();
        }
    }

    /// Add the insert location if it is among the top three insertion
    /// locations.
    ///
    /// The three best insertion locations are stored in ascending order
    /// on cost such that the best is first
    pub fn add(&mut self, loc: InsertLocation) {
        if loc.cost > self.locations[2].cost {
            return;
        } else if loc.cost > self.locations[1].cost {
            self.locations[2] = loc;
        } else if loc.cost > self.locations[0].cost {
            self.locations[2] = self.locations[1];
            self.locations[1] = loc;
        } else {
            self.locations[2] = self.locations[1];
            self.locations[1] = self.locations[0];
            self.locations[0] = loc;
        }
    }
}
