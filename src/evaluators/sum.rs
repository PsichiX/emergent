use crate::{consideration::*, Scalar};

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
