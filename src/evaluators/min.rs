//! Calculates minimum of sub-consideratiosn scores.

use crate::{consideration::*, Scalar};

/// Gives minimum of all considerations scores.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = EvaluatorMin::default()
///     .consideration(1.0)
///     .consideration(2.0);
/// assert_eq!(consideration.score(&()), 1.0);
/// ```
pub struct EvaluatorMin<M> {
    pub considerations: Vec<Box<dyn Consideration<M>>>,
}

impl<M> Default for EvaluatorMin<M> {
    fn default() -> Self {
        Self {
            considerations: vec![],
        }
    }
}

impl<M> EvaluatorMin<M> {
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

impl<M> Consideration<M> for EvaluatorMin<M> {
    fn score(&self, memory: &M) -> Scalar {
        self.considerations
            .iter()
            .map(|consideration| consideration.score(memory))
            .min_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or_default()
    }
}
