use crate::models::IntType;

// Used to represent the polar angles as integers
const MAX_ANGLE: IntType = 65536;

/// Represents the sector with start and end polar angles in a circle
#[derive(Clone, Debug)]
pub struct CircleSector {
    pub start: IntType,
    pub end: IntType,
}

impl CircleSector {
    pub fn new() -> Self {
        Self { start: 0, end: 0 }
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub fn from_angle(&mut self, angle: IntType) {
        self.start = angle;
        self.end = angle;
    }

    pub fn extend(&mut self, angle: IntType) {
        if self.start == 0 && self.end == 0 {
            self.from_angle(angle);
        } else {
            if !self.is_enclosed(angle) {
                if (angle - self.end).rem_euclid(MAX_ANGLE)
                    <= (self.start - angle).rem_euclid(MAX_ANGLE)
                {
                    self.end = angle;
                } else {
                    self.start = angle;
                }
            }
        }
    }

    pub fn is_enclosed(&self, angle: IntType) -> bool {
        (angle - self.start).rem_euclid(MAX_ANGLE) <= (self.end - self.start).rem_euclid(MAX_ANGLE)
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        (other.start - self.start).rem_euclid(MAX_ANGLE)
            <= (self.end - self.start).rem_euclid(MAX_ANGLE)
            || (self.start - other.start).rem_euclid(MAX_ANGLE)
                <= (other.end - other.start).rem_euclid(MAX_ANGLE)
    }
}
