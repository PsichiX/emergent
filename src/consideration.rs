//! Considerations are used to tell possibility, how important certain fact about the world is.
//!
//! See [`Consideration`] for more info about considerations.
//!
//! See [`crate::evaluators`] for more info about evaluators (operations on sets of considerations).

use crate::{condition::*, score_mapping::*, Scalar};

/// Consideration represent the score (also called weight, possibility, likeliness) of certain fact
/// about the state of the world.
///
/// Imagine memory stores information about number of bannanas in the backpack, and you want to know
/// how important for you is to get new one to not get hungry during the day - the more bannanas you
/// have, the less likely it is for you to end up hungry and that score is used to decide if you need
/// to go and get new one or even estimate how many of them you would need.
///
/// User should make considerations as lightweight and as small as possible. The reason for that is
/// to make them reused and combined into bigger sets of considerations using evaluators
/// ([`crate::evaluators`]).
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// struct Hunger;
///
/// impl Consideration<Memory> for Hunger {
///     fn score(&self, memory: &Memory) -> Scalar {
///         1.0 / memory.counter as Scalar
///     }
/// }
///
/// let mut memory = Memory { counter: 10 };
/// assert_eq!(Hunger.score(&memory), 0.1);
/// ```
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

/// Consideration that wraps a closure.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// let mut memory = Memory { counter: 10 };
/// let consideration = ClosureConsideration::new(
///     |memory: &Memory| 1.0 / memory.counter as Scalar,
/// );
/// assert_eq!(consideration.score(&memory), 0.1);
/// ```
pub struct ClosureConsideration<M = ()>(pub Box<dyn Fn(&M) -> Scalar>);

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

impl<M> std::fmt::Debug for ClosureConsideration<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureConsideration").finish()
    }
}

/// Consideration uses [`ScoreMapping`] to remap score of wrapped consideration.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// struct Hunger;
///
/// impl Consideration<Memory> for Hunger {
///     fn score(&self, memory: &Memory) -> Scalar {
///         1.0 / memory.counter as Scalar
///     }
/// }
///
/// let mut memory = Memory { counter: 10 };
/// let consideration = ConsiderationRemap::new(Hunger, ReverseScoreMapping);
/// assert_eq!(consideration.score(&memory), 0.9);
/// ```
pub struct ConsiderationRemap<M = (), T = NoScoreMapping>
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

impl<M, T> std::fmt::Debug for ConsiderationRemap<M, T>
where
    T: ScoreMapping + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsiderationRemap")
            .field("mapping", &self.mapping)
            .finish()
    }
}

/// Consideration wraps [`Condition`] and converts it into consideration.
///
/// If condition returns true, it gives score of [`ConditionConsideration::positive`], if not then
/// it gives [`ConditionConsideration::negative`].
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// let mut memory = Memory { counter: 1 };
/// let consideration = ConditionConsideration::new(
///     ClosureCondition::new(|memory: &Memory| memory.counter > 0),
///     1.0,
///     0.0,
/// );
/// assert_eq!(consideration.score(&memory), 1.0);
/// ```
pub struct ConditionConsideration<M = ()> {
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

    pub fn unit<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self {
            condition: Box::new(condition),
            positive: 1.0,
            negative: 0.0,
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

impl<M> std::fmt::Debug for ConditionConsideration<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionConsideration")
            .field("positive", &self.positive)
            .field("negative", &self.negative)
            .finish()
    }
}
