use crate::{condition::*, score_mapping::*, Scalar};

pub trait Consideration<M = ()> {
    fn score(&self, memory: &M) -> Scalar;
}

impl<M> Consideration<M> for dyn Fn(&M) -> Scalar {
    fn score(&self, memory: &M) -> Scalar {
        self(memory)
    }
}

impl<M> Consideration<M> for Scalar {
    fn score(&self, _: &M) -> Scalar {
        *self
    }
}

pub struct ClosureConsideration<M>(Box<dyn Fn(&M) -> Scalar>);

impl<M> ClosureConsideration<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&M) -> Scalar + 'static,
    {
        Self(Box::new(f))
    }
}

impl<M> Consideration<M> for ClosureConsideration<M> {
    fn score(&self, memory: &M) -> Scalar {
        (self.0)(memory)
    }
}

pub struct ConsiderationConstant(pub Scalar);

impl<M> Consideration<M> for ConsiderationConstant {
    fn score(&self, _: &M) -> Scalar {
        self.0
    }
}

pub struct ConsiderationRemap<M, T = NoScoreMapping>
where
    T: ScoreMapping,
{
    pub consideration: Box<dyn Consideration<M>>,
    pub mapping: T,
}

impl<M, T> ConsiderationRemap<M, T>
where
    T: ScoreMapping,
{
    pub fn new<C>(consideration: C, mapping: T) -> Self
    where
        C: Consideration<M> + 'static,
    {
        Self {
            consideration: Box::new(consideration),
            mapping,
        }
    }
}

impl<M, T> Consideration<M> for ConsiderationRemap<M, T>
where
    T: ScoreMapping,
{
    fn score(&self, memory: &M) -> Scalar {
        self.mapping.remap(self.consideration.score(memory))
    }
}

pub struct ConditionConsideration<M> {
    pub condition: Box<dyn Condition<M>>,
    pub positive: Scalar,
    pub negative: Scalar,
}

impl<M> ConditionConsideration<M> {
    pub fn new<C>(condition: C, positive: Scalar, negative: Scalar) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self {
            condition: Box::new(condition),
            positive,
            negative,
        }
    }
}

impl<M> Consideration<M> for ConditionConsideration<M> {
    fn score(&self, memory: &M) -> Scalar {
        if self.condition.validate(memory) {
            self.positive
        } else {
            self.negative
        }
    }
}
