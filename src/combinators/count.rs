//! Tests if certain number of sub-conditions succeeds.

use crate::condition::*;
use std::ops::Range;

/// Defines a range bound.
#[derive(Debug, Copy, Clone)]
pub enum CombinatorCountBound {
    /// None bound means infinity.
    None,
    /// Exclusive bound (less/greater than N).
    Exclusive(usize),
    /// Inclusive bound (less/greater than or equal to N).
    Inclusive(usize),
}

impl CombinatorCountBound {
    /// Tests if value passes when `self` used as lower bound.
    pub fn validate_lower(self, count: usize) -> bool {
        match self {
            Self::None => true,
            Self::Exclusive(v) => count > v,
            Self::Inclusive(v) => count >= v,
        }
    }

    /// Tests if value passes when `self` used as upper bound.
    pub fn validate_upper(self, count: usize) -> bool {
        match self {
            Self::None => true,
            Self::Exclusive(v) => count < v,
            Self::Inclusive(v) => count <= v,
        }
    }
}

/// Returns `true` if number of passing conditions is in bounds range.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let condition = CombinatorCount::new(CombinatorCountBound::None..CombinatorCountBound::None)
///     .condition(false)
///     .condition(true);
/// assert_eq!(condition.validate(&()), true);
///
/// let condition = CombinatorCount::new(CombinatorCountBound::None..CombinatorCountBound::Inclusive(2))
///     .condition(true)
///     .condition(true);
/// assert_eq!(condition.validate(&()), true);
/// ```
pub struct CombinatorCount<M> {
    pub conditions: Vec<Box<dyn Condition<M>>>,
    pub bounds: Range<CombinatorCountBound>,
}

impl<M> CombinatorCount<M> {
    /// Constructs new condition with bounds.
    pub fn new(bounds: Range<CombinatorCountBound>) -> Self {
        Self {
            conditions: vec![],
            bounds,
        }
    }

    /// Adds sub-condition.
    pub fn condition<C>(mut self, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl<M> Condition<M> for CombinatorCount<M> {
    fn validate(&self, memory: &M) -> bool {
        let count = self
            .conditions
            .iter()
            .filter(|condition| condition.validate(memory))
            .count();
        self.bounds.start.validate_lower(count) && self.bounds.end.validate_upper(count)
    }
}

impl<M> std::fmt::Debug for CombinatorCount<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombinatorCount")
            .field("bounds", &self.bounds)
            .finish()
    }
}
