use std::borrow::Borrow;

const EPSILON: f64 = 0.000001;

/// Extension trait enabling float comparison using an epsilon value
pub trait FloatCompare<T> {
    fn approx_lt(&self, other: T) -> bool;
    fn approx_lte(&self, other: T) -> bool;
    fn approx_gt(&self, other: T) -> bool;
    fn approx_gte(&self, other: T) -> bool;
    fn approx_eq(&self, other: T) -> bool;
}

impl<T: Borrow<f64>> FloatCompare<T> for f64 {
    fn approx_lt(&self, other: T) -> bool {
        *self < other.borrow() - EPSILON
    }

    fn approx_lte(&self, other: T) -> bool {
        self < other.borrow() || self.approx_eq(other)
    }

    fn approx_gt(&self, other: T) -> bool {
        *self > other.borrow() + EPSILON
    }

    fn approx_gte(&self, other: T) -> bool {
        self > other.borrow() || self.approx_eq(other)
    }

    fn approx_eq(&self, other: T) -> bool {
        (self - other.borrow()).abs() < EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_approx_lt() {
        assert_eq!(false, 10.0.approx_lt(9.0));
        assert_eq!(false, 10.0.approx_lt(9.9999));
        assert_eq!(false, 10.0.approx_lt(9.999999));
        assert_eq!(false, 10.0.approx_lt(10.0));
        assert_eq!(false, 10.0.approx_lt(10.0000001));
        assert_eq!(true, 10.0.approx_lt(10.00001));
        assert_eq!(true, 10.0.approx_lt(11.0));
    }

    #[test]
    fn f64_approx_lte() {
        assert_eq!(false, 10.0.approx_lte(9.0));
        assert_eq!(false, 10.0.approx_lte(9.9999));
        assert_eq!(true, 10.0.approx_lte(9.999999));
        assert_eq!(true, 10.0.approx_lte(10.0));
        assert_eq!(true, 10.0.approx_lte(10.0000001));
        assert_eq!(true, 10.0.approx_lte(10.00001));
        assert_eq!(true, 10.0.approx_lte(11.0));
    }

    #[test]
    fn f64_approx_gt() {
        assert_eq!(true, 10.0.approx_gt(9.0));
        assert_eq!(true, 10.0.approx_gt(9.9999));
        assert_eq!(false, 10.0.approx_gt(9.999999));
        assert_eq!(false, 10.0.approx_gt(10.0));
        assert_eq!(false, 10.0.approx_gt(10.0000001));
        assert_eq!(false, 10.0.approx_gt(10.00001));
        assert_eq!(false, 10.0.approx_gt(11.0));
    }

    #[test]
    fn f64_approx_gte() {
        assert_eq!(true, 10.0.approx_gte(9.0));
        assert_eq!(true, 10.0.approx_gte(9.9999));
        assert_eq!(true, 10.0.approx_gte(9.999999));
        assert_eq!(true, 10.0.approx_gte(10.0));
        assert_eq!(true, 10.0.approx_gte(10.0000001));
        assert_eq!(false, 10.0.approx_gte(10.00001));
        assert_eq!(false, 10.0.approx_gte(11.0));
    }

    #[test]
    fn f64_approx_eq() {
        assert_eq!(false, 10.0.approx_eq(9.0));
        assert_eq!(false, 10.0.approx_eq(9.9999));
        assert_eq!(true, 10.0.approx_eq(9.999999));
        assert_eq!(true, 10.0.approx_eq(10.0));
        assert_eq!(true, 10.0.approx_eq(10.0000001));
        assert_eq!(false, 10.0.approx_eq(10.00001));
        assert_eq!(false, 10.0.approx_eq(11.0));
    }
}
