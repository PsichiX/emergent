use crate::{consideration::*, Scalar};

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
            .max_by(|a, b| a.partial_cmp(&b).unwrap())
            .unwrap_or_default()
    }
}
