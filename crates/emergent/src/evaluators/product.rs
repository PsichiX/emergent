//! Calculates product of sub-consideratiosn scores.

use crate::{consideration::*, Scalar};

/// Gives product of all considerations scores.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let consideration = EvaluatorProduct::default()
///     .consideration(5.0)
///     .consideration(4.0);
/// assert_eq!(consideration.score(&()), 20.0);
/// ```
pub struct EvaluatorProduct<M> {
    pub considerations: Vec<Box<dyn Consideration<M>>>,
}

impl<M> Default for EvaluatorProduct<M> {
    fn default() -> Self {
        Self {
            considerations: vec![],
        }
    }
}

impl<M> EvaluatorProduct<M> {
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

impl<M> Consideration<M> for EvaluatorProduct<M> {
    fn score(&self, memory: &M) -> Scalar {
        self.considerations
            .iter()
            .map(|consideration| consideration.score(memory))
            .product()
    }
}

impl<M> std::fmt::Debug for EvaluatorProduct<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvaluatorProduct").finish()
    }
}
