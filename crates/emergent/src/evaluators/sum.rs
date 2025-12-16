//! Calculates sum of sub-consideratiosn scores.

use crate::{Scalar, consideration::*};

/// Gives sum of all considerations scores.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = EvaluatorSum::default()
///     .consideration(40.0)
///     .consideration(2.0);
/// assert_eq!(consideration.score(&()), 42.0);
/// ```
pub struct EvaluatorSum<M> {
    pub considerations: Vec<Box<dyn Consideration<M>>>,
}

impl<M> Default for EvaluatorSum<M> {
    fn default() -> Self {
        Self {
            considerations: vec![],
        }
    }
}

impl<M> EvaluatorSum<M> {
    /// Constructs new consideration wih list of sub-considerations.
    pub fn new(considerations: Vec<Box<dyn Consideration<M>>>) -> Self {
        Self { considerations }
    }

    /// Add child consideration.
    pub fn consideration<C>(mut self, consideration: C) -> Self
    where
        C: Consideration<M> + 'static,
    {
        self.considerations.push(Box::new(consideration));
        self
    }
}

impl<M> Consideration<M> for EvaluatorSum<M> {
    fn score(&self, memory: &M) -> Scalar {
        self.considerations
            .iter()
            .map(|consideration| consideration.score(memory))
            .sum()
    }
}

impl<M> std::fmt::Debug for EvaluatorSum<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluatorSum").finish()
    }
}
