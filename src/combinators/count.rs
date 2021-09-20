use crate::condition::*;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub enum CombinatorCountBound {
    None,
    Exclusive(usize),
    Inclusive(usize),
}

impl CombinatorCountBound {
    pub fn validate_lower(self, count: usize) -> bool {
        match self {
            Self::None => true,
            Self::Exclusive(v) => count > v,
            Self::Inclusive(v) => count >= v,
        }
    }

    pub fn validate_upper(self, count: usize) -> bool {
        match self {
            Self::None => true,
            Self::Exclusive(v) => count < v,
            Self::Inclusive(v) => count <= v,
        }
    }
}

pub struct CombinatorCount<M> {
    pub conditions: Vec<Box<dyn Condition<M>>>,
    pub expectation: Range<CombinatorCountBound>,
}

impl<M> CombinatorCount<M> {
    pub fn new(expectation: Range<CombinatorCountBound>) -> Self {
        Self {
            conditions: vec![],
            expectation,
        }
    }

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
        self.expectation.start.validate_lower(count) && self.expectation.end.validate_upper(count)
    }
}
