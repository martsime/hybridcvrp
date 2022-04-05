use std::fmt;
use std::ptr;

use crate::models::CircleSector;

#[derive(Clone, Debug)]
pub struct LinkNode {
    pub number: usize,
    pub angle: i32,
    pub successor: *mut LinkNode,
    pub predecessor: *mut LinkNode,
    pub route: *mut LinkRoute,
    pub position: usize,
    pub last_tested: i32,
    pub cum_distance: f64,
    pub cum_load: f64,
    // Change in distance when removing the node
    pub delta_removal: f64,
}

impl LinkNode {
    pub unsafe fn new(number: usize, angle: i32) -> Self {
        Self {
            number,
            angle,
            successor: ptr::null_mut(),
            predecessor: ptr::null_mut(),
            route: ptr::null_mut(),
            position: 0,
            last_tested: 0,
            cum_distance: f64::default(),
            cum_load: f64::default(),
            delta_removal: f64::default(),
        }
    }

    pub fn is_depot(&self) -> bool {
        self.number == 0
    }

    /// Links together two nodes
    #[inline]
    pub unsafe fn link_nodes(node_one: *mut LinkNode, node_two: *mut LinkNode) {
        (*node_one).successor = node_two;
        (*node_two).predecessor = node_one;
    }

    /// Insert `node_one` directly after `node_two`
    #[inline]
    pub unsafe fn insert_node(node_one: *mut LinkNode, node_two: *mut LinkNode) {
        let node_one_prev = (*node_one).predecessor;
        let node_one_next = (*node_one).successor;
        let node_two_next = (*node_two).successor;
        Self::link_nodes(node_one_prev, node_one_next);
        Self::link_nodes(node_two, node_one);
        Self::link_nodes(node_one, node_two_next);
    }

    /// Reverse the sequence going forward from `from_node`
    pub unsafe fn forward_reverse(
        mut from_node: *mut LinkNode,
        to_node: *mut LinkNode,
        new_first_node: *mut LinkNode,
    ) {
        let mut node = (*from_node).successor;
        while !node.is_null() {
            let next_node = (*node).successor;

            // If at first node and we have a new first node
            if next_node.is_null() && !new_first_node.is_null() {
                Self::link_nodes(new_first_node, from_node);
            } else {
                Self::link_nodes(node, from_node);
            }
            if !to_node.is_null() {
                if (*node).number == (*to_node).number {
                    break;
                }
            }
            from_node = node;
            node = next_node;
        }
    }

    /// Reverse the sequence going backward from `from_node`
    pub unsafe fn backward_reverse(
        mut from_node: *mut LinkNode,
        to_node: *mut LinkNode,
        new_last_node: *mut LinkNode,
    ) {
        let mut node = (*from_node).predecessor;
        while !node.is_null() {
            let next_node = (*node).predecessor;

            // If at last node and we have a new last node
            if next_node.is_null() && !new_last_node.is_null() {
                Self::link_nodes(from_node, new_last_node);
            } else {
                Self::link_nodes(from_node, node);
            }
            if !to_node.is_null() {
                if (*node).number == (*to_node).number {
                    break;
                }
            }
            from_node = node;
            node = next_node;
        }
    }

    #[inline]
    pub unsafe fn replace_end_depot(mut from_node: *mut LinkNode, end_depot: *mut LinkNode) {
        let mut next_node = (*from_node).successor;
        while !next_node.is_null() {
            // Update last depot when at last node
            if (*next_node).successor.is_null() {
                Self::link_nodes(from_node, end_depot);
            }
            from_node = next_node;
            next_node = (*next_node).successor;
        }
    }
}

#[derive(Clone, Debug)]
pub struct LinkRoute {
    pub index: usize,

    // Reference to the depot nodes
    pub start_depot: *mut LinkNode,
    pub end_depot: *mut LinkNode,

    // Number of customers visisted in the route
    pub num_customers: usize,

    // Used keep track of changes
    pub last_modified: i32,

    // Used keep track of changes
    pub last_tested_swap_star: i32,

    // Circle sector of the route
    pub sector: CircleSector,

    // Distance of the route
    pub distance: f64,

    // Total load on the route
    pub load: f64,

    // Total overload on the route
    pub overload: f64,

    // Penalized cost
    pub cost: f64,
}

impl LinkRoute {
    pub fn new(index: usize, start_depot: *mut LinkNode, end_depot: *mut LinkNode) -> Self {
        Self {
            index,
            start_depot,
            end_depot,
            num_customers: 0,
            last_modified: 0,
            last_tested_swap_star: 0,
            sector: CircleSector::new(),
            distance: f64::MAX,
            load: f64::MAX,
            overload: f64::MAX,
            cost: f64::INFINITY,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.num_customers == 0
    }
}

impl fmt::Display for LinkRoute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut customers: Vec<usize> = Vec::new();
        let mut next_node_ptr = self.start_depot;
        while !next_node_ptr.is_null() {
            unsafe {
                let node = &*next_node_ptr;
                if !node.is_depot() {
                    customers.push(node.number);
                }
                next_node_ptr = node.successor;
            }
        }
        write!(f, "{:?}", customers)
    }
}
