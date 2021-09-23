//! Tests if all sub-conditions succeeds.

use crate::condition::*;

/// Returns `true` if all of its conditions return `true`.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let condition = CombinatorAll::default()
///     .condition(true)
///     .condition(true);
/// assert_eq!(condition.validate(&()), true);
///
/// let condition = CombinatorAll::default()
///     .condition(false)
///     .condition(true);
/// assert_eq!(condition.validate(&()), false);
/// ```
pub struct CombinatorAll<M> {
    pub conditions: Vec<Box<dyn Condition<M>>>,
}

impl<M> Default for CombinatorAll<M> {
    fn default() -> Self {
        Self { conditions: vec![] }
    }
}

impl<M> CombinatorAll<M> {
    /// Constructs new condition with list of sub-conditions.
    pub fn new(conditions: Vec<Box<dyn Condition<M>>>) -> Self {
        Self { conditions }
    }

    /// Add child condition.
    pub fn condition<C>(mut self, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl<M> Condition<M> for CombinatorAll<M> {
    fn validate(&self, memory: &M) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.validate(memory))
    }
}
