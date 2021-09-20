use crate::{consideration::*, Scalar};

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
