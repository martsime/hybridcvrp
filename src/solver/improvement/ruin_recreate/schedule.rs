use std::time::{Duration, Instant};

use crate::utils::{FloatCompare, Random};

const COOLING_FACTOR: f64 = 2.0;

// MIN_VALUE = e^(-COOLING_FACTOR)
const MIN_VALUE: f64 = 0.1353352832366127f64;

pub trait Acceptance {
    fn accept(&self, new_cost: f64, old_cost: f64, random: &Random) -> bool;

    fn update(&mut self);

    fn completed(&self) -> bool;

    fn reset(&mut self);

    fn print(&self);
}

pub trait TemperatureAcceptance: Acceptance {
    fn temp(&self) -> f64;

    #[inline]
    fn accept(&self, new_cost: f64, old_cost: f64, random: &Random) -> bool {
        new_cost.approx_lt(old_cost - self.temp() * random.real().ln())
    }

    /// Returns a value between 0.0 and 1.0
    fn elapsed(&self) -> f64;
}

/// Iteration based schedule using exponential decay in temperature
pub struct IterationSchedule {
    temp: f64,
    start_temp: f64,
    total_iterations: usize,
    pub iteration: usize,
}

impl IterationSchedule {
    pub fn new(start_temp: f64, iterations: usize) -> Self {
        Self {
            temp: start_temp,
            start_temp,
            total_iterations: iterations,
            iteration: 0,
        }
    }
}

impl Acceptance for IterationSchedule {
    #[inline]
    fn accept(&self, new_cost: f64, old_cost: f64, random: &Random) -> bool {
        <Self as TemperatureAcceptance>::accept(&self, new_cost, old_cost, random)
    }

    fn update(&mut self) {
        self.iteration += 1;
        let t = self.elapsed();
        self.temp = ((t * -COOLING_FACTOR).exp() - t * MIN_VALUE) * self.start_temp;
    }

    fn completed(&self) -> bool {
        self.iteration >= self.total_iterations
    }

    fn reset(&mut self) {
        self.iteration = 0;
        self.temp = self.start_temp;
    }

    fn print(&self) {
        log::info!("Total iterations: {}", self.total_iterations);
    }
}

impl TemperatureAcceptance for IterationSchedule {
    fn temp(&self) -> f64 {
        self.temp
    }

    fn elapsed(&self) -> f64 {
        self.iteration as f64 / self.total_iterations as f64
    }
}

/// Time based schedule using exponential decay in temperature
pub struct TimeSchedule {
    start: Instant,
    duration: f64,
    temp: f64,
    start_temp: f64,
    pub iterations: usize,
    iterations_since_update: usize,
    update_rate: usize,
    completed: bool,
}

impl TimeSchedule {
    pub fn new(start_temp: f64, duration: Duration) -> Self {
        Self {
            /// The duration of the schedule
            duration: duration.as_secs_f64(),

            /// The start temperature
            start_temp,

            /// The start time
            start: Instant::now(),

            /// The temperature is update every `update_rate` iteration
            update_rate: 100,

            /// Number of iterations since the last update
            iterations_since_update: 0,

            // The number of iterations performed
            iterations: 0,

            /// The current temperature
            temp: start_temp,

            completed: false,
        }
    }

    pub fn set_update_rate(&mut self, rate: usize) {
        self.update_rate = rate;
    }
}

impl Acceptance for TimeSchedule {
    #[inline]
    fn accept(&self, new_cost: f64, old_cost: f64, random: &Random) -> bool {
        <Self as TemperatureAcceptance>::accept(&self, new_cost, old_cost, random)
    }

    fn update(&mut self) {
        self.iterations_since_update += 1;
        self.iterations += 1;
        if self.iterations_since_update == self.update_rate {
            let t = self.elapsed();
            self.temp = ((t * -COOLING_FACTOR).exp() - t * MIN_VALUE) * self.start_temp;
            self.iterations_since_update = 0;
            if t >= 1.0 {
                self.completed = true;
            }
        }
    }

    fn completed(&self) -> bool {
        self.completed
    }

    fn reset(&mut self) {
        self.start = Instant::now();
        self.iterations = 0;
        self.iterations_since_update = 0;
    }

    fn print(&self) {
        log::info!("Duration: {:?}", self.duration);
    }
}

impl TemperatureAcceptance for TimeSchedule {
    fn temp(&self) -> f64 {
        self.temp
    }

    fn elapsed(&self) -> f64 {
        self.start.elapsed().as_secs_f64() / self.duration
    }
}

pub enum AcceptanceCriterion {
    Iteration(IterationSchedule),
    Time(TimeSchedule),
}

impl Acceptance for AcceptanceCriterion {
    #[inline]
    fn accept(&self, new_cost: f64, old_cost: f64, random: &Random) -> bool {
        match self {
            Self::Iteration(schedule) => <IterationSchedule as TemperatureAcceptance>::accept(
                schedule, new_cost, old_cost, random,
            ),
            Self::Time(schedule) => <TimeSchedule as TemperatureAcceptance>::accept(
                schedule, new_cost, old_cost, random,
            ),
        }
    }

    fn update(&mut self) {
        match self {
            Self::Iteration(schedule) => schedule.update(),
            Self::Time(schedule) => schedule.update(),
        }
    }

    fn completed(&self) -> bool {
        match self {
            Self::Iteration(schedule) => schedule.completed(),
            Self::Time(schedule) => schedule.completed(),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Iteration(schedule) => schedule.reset(),
            Self::Time(schedule) => schedule.reset(),
        }
    }

    fn print(&self) {
        match self {
            Self::Iteration(schedule) => schedule.print(),
            Self::Time(schedule) => schedule.print(),
        }
    }
}

impl From<IterationSchedule> for AcceptanceCriterion {
    fn from(schedule: IterationSchedule) -> Self {
        Self::Iteration(schedule)
    }
}

impl From<TimeSchedule> for AcceptanceCriterion {
    fn from(schedule: TimeSchedule) -> Self {
        Self::Time(schedule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::improvement::{IterationSchedule, TimeSchedule};
    use std::thread;

    #[test]
    fn iteration_schedule() {
        let mut schedule = IterationSchedule::new(100.0, 1000);

        for _ in 0..100 {
            schedule.update();
        }

        // n = 0.1: (e^(-2n) - n*e^(-2)) * 100 = 80.51972247543206
        assert!(schedule.temp().approx_eq(80.51972247543206));

        for _ in 0..400 {
            schedule.update()
        }
        // n = 0.5: (e^(-2n) - n*e^(-2)) * 100 = 30.0211799553136
        assert!(schedule.temp().approx_eq(30.0211799553136));

        for _ in 0..400 {
            schedule.update()
        }

        // n = 0.9: (e^(-2n) - n*e^(-2)) * 100 = 4.34971333086351
        assert!(schedule.temp().approx_eq(4.34971333086351));
    }

    #[test]
    fn time_schedule() {
        let mut schedule = TimeSchedule::new(100.0, Duration::from_millis(100));
        schedule.update_rate = 1;
        thread::sleep(Duration::from_millis(1));
        schedule.update();
        // assert!(100.0.approx_eq(schedule.temp()));
        assert_eq!(100.0, schedule.temp());

        thread::sleep(Duration::from_millis(50));
        assert!(10.0.approx_eq(schedule.temp()));

        thread::sleep(Duration::from_millis(40));
        assert!(1.0.approx_eq(schedule.temp()));
    }
}
