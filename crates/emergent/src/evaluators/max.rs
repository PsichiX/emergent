//! Calculates maximum of sub-consideratiosn scores.

use crate::{Scalar, consideration::*};

/// Gives maximum of all considerations scores.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = EvaluatorMax::default()
///     .consideration(1.0)
///     .consideration(2.0);
/// assert_eq!(consideration.score(&()), 2.0);
/// ```
pub struct EvaluatorMax<M> {
    pub considerations: Vec<Box<dyn Consideration<M>>>,
}

impl<M> Default for EvaluatorMax<M> {
    fn default() -> Self {
        Self {
            considerations: vec![],
        }
    }
}

impl<M> EvaluatorMax<M> {
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

impl<M> Consideration<M> for EvaluatorMax<M> {
    fn score(&self, memory: &M) -> Scalar {
        self.considerations
            .iter()
            .map(|consideration| consideration.score(memory))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or_default()
    }
}

impl<M> std::fmt::Debug for EvaluatorMax<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluatorMax").finish()
    }
}
