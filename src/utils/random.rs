use fastrand::Rng;
use std::cell::RefCell;

use ahash::RandomState;

#[derive(Debug)]
pub struct Random {
    pub rng: RefCell<Rng>,
    pub seed: Option<u64>,
}

impl Random {
    pub fn new() -> Self {
        Self {
            rng: RefCell::new(Rng::new()),
            seed: None,
        }
    }

    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: RefCell::new(Rng::with_seed(seed)),
            seed: Some(seed),
        }
    }

    pub fn shuffle<T>(&self, container: &mut [T]) {
        self.rng.borrow_mut().shuffle(container);
    }

    pub fn real(&self) -> f64 {
        self.rng.borrow_mut().f64() as f64
    }

    // Get random number in range [lower, upper). Upper is not inclusive
    pub fn range_usize(&self, lower: usize, upper: usize) -> usize {
        self.rng.borrow_mut().usize(lower..upper)
    }

    // Sample `number` elements from the vec
    pub fn sample_from_vec<T>(&self, mut vec: Vec<T>, number: usize) -> Vec<T> {
        // The vec must have more elements than are beign sampled
        assert_eq!(true, vec.len() >= number);
        let mut new_vec = Vec::with_capacity(number);

        // Take an element from the vec until `number` elements have been sampled
        while new_vec.len() < number {
            new_vec.push(vec.remove(self.rng.borrow_mut().usize(0..vec.len())));
        }
        new_vec
    }

    pub fn reset(&self) {
        if let Some(seed) = self.seed {
            self.rng.replace(Rng::with_seed(seed));
        } else {
            self.rng.replace(Rng::new());
        }
    }

    pub fn random_state(&self) -> RandomState {
        if let Some(seed) = self.seed {
            RandomState::with_seeds(seed, seed + 123, seed + 321, seed + 1337)
        } else {
            RandomState::new()
        }
    }
}
