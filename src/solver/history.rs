use instant::Instant;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::solver::genetic::Individual;
use crate::solver::Context;

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoricSolution {
    // Routes in the solution
    pub routes: Vec<Vec<usize>>,

    // Cost of solution
    pub cost: f64,
}

impl From<&Individual> for HistoricSolution {
    fn from(individual: &Individual) -> Self {
        Self {
            routes: individual.phenotype.clone(),
            cost: individual.penalized_cost(),
        }
    }
}

impl fmt::Display for HistoricSolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut route_number = 1;
        for route in self.routes.iter() {
            if route.len() > 0 {
                let mut route_string = format!("Route #{}:", route_number);
                for stop in route.iter() {
                    route_string.push_str(&format!(" {}", stop));
                }
                route_number += 1;
                writeln!(f, "{}", route_string)?;
            }
        }
        write!(f, "Cost {}", self.cost.round() as u64)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct HistoryEntry {
    pub solution: HistoricSolution,
    // Timestamp in duration since solver started
    pub timestamp: Duration,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryMessage {
    pub timestamp: Duration,
    pub message: String,
}

impl fmt::Display for HistoryMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Time: {:?}, {}", self.timestamp, self.message)
    }
}

#[derive(Debug)]
pub struct SearchHistory {
    // History of solutions
    history: Vec<HistoryEntry>,

    pub best_cost: f64,

    // Timestamp of when the solver started
    pub start_time: Instant,

    pub messages: Vec<HistoryMessage>,
}

impl SearchHistory {
    pub fn new(start_time: Instant) -> Self {
        Self {
            history: Vec::new(),
            best_cost: f64::INFINITY,
            start_time,
            messages: Vec::new(),
        }
    }

    fn remove_previous_data(&mut self) {
        if let Some(last) = self.history.last_mut() {
            last.solution.routes = Vec::new();
        }
    }

    pub fn add(&mut self, _ctx: &Context, individual: &Individual) {
        self.best_cost = individual.penalized_cost();
        let timestamp = self.start_time.elapsed();
        let history_entry = HistoryEntry {
            solution: HistoricSolution::from(individual),
            timestamp,
        };

        #[cfg(feature = "dimacs")]
        println!("{}", history_entry.solution);

        let new_best_message = HistoryMessage {
            message: format!("New best: {:?}", self.best_cost),
            timestamp,
        };

        log::info!("{}", new_best_message);

        // Keep only json for the last found solution
        self.remove_previous_data();
        self.history.push(history_entry);
    }

    pub fn add_message(&mut self, message: String) {
        let history_message = HistoryMessage {
            message,
            timestamp: self.start_time.elapsed(),
        };
        self.messages.push(history_message);
    }

    pub fn entries(&self) -> &Vec<HistoryEntry> {
        &self.history
    }

    pub fn last_entry(&self) -> Option<&HistoryEntry> {
        self.history.last()
    }
}
