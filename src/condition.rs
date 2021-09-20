use crate::{consideration::*, Scalar};

pub trait Condition<M> {
    fn validate(&self, memory: &M) -> bool;
}

impl<M> Condition<M> for dyn Fn(&M) -> bool {
    fn validate(&self, memory: &M) -> bool {
        self(memory)
    }
}

impl<M> Condition<M> for bool {
    fn validate(&self, _: &M) -> bool {
        *self
    }
}

pub struct ClosureCondition<M>(Box<dyn Fn(&M) -> bool>);

impl<M> ClosureCondition<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&M) -> bool + 'static,
    {
        Self(Box::new(f))
    }
}

impl<M> Condition<M> for ClosureCondition<M> {
    fn validate(&self, memory: &M) -> bool {
        (self.0)(memory)
    }
}

pub struct ConditionConstant(pub bool);

impl<M> Condition<M> for ConditionConstant {
    fn validate(&self, _: &M) -> bool {
        self.0
    }
}

pub struct ConditionInvert<M>(pub Box<dyn Condition<M>>);

impl<M> ConditionInvert<M> {
    pub fn new<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self(Box::new(condition))
    }
}

impl<M> Condition<M> for ConditionInvert<M> {
    fn validate(&self, memory: &M) -> bool {
        !self.0.validate(memory)
    }
}

pub struct ConsiderationCondition<M> {
    pub consideration: Box<dyn Consideration<M>>,
    pub threshold: Scalar,
}

impl<M> ConsiderationCondition<M> {
    pub fn new<C>(consideration: C, threshold: Scalar) -> Self
    where
        C: Consideration<M> + 'static,
    {
        Self {
            consideration: Box::new(consideration),
            threshold,
        }
    }
}

impl<M> Condition<M> for ConsiderationCondition<M> {
    fn validate(&self, memory: &M) -> bool {
        self.consideration.score(memory) > self.threshold
    }
}
