//! Conditions serve purpose of a boolean query to the memory state.
//!
//! See [`Condition`] for more info about conditions.
//!
//! See [`crate::combinators`] for more info about combinators (operations on sets of conditions).

use crate::{consideration::*, Scalar};

/// Condition represent the simplest question about the state of the world via provided memory.
///
/// Imagine memory stores information about number of bannanas in the backpack, and you want to know
/// if there are at least 3 so you can use that information to decide if you need to find more of
/// them to not get hungry during the day.
///
/// User should make conditions as lightweight and as small as possible. The reason for that is to
/// make them reused and combined into bigger sets of conditions using combinators
/// ([`crate::combinators`]).
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// struct AboveThreshold(usize);
///
/// impl Condition<Memory> for AboveThreshold {
///     fn validate(&self, memory: &Memory) -> bool {
///         memory.counter > self.0
///     }
/// }
///
/// let mut memory = Memory { counter: 1 };
/// assert!(AboveThreshold(0).validate(&memory));
/// ```
pub trait Condition<M = ()> {
    /// Tells if given condition is met based on the state of the memory provided.
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

/// Condition that wraps a closure.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// let mut memory = Memory { counter: 1 };
/// let condition = ClosureCondition::new(|memory: &Memory| memory.counter > 0);
/// assert!(condition.validate(&memory));
/// ```
pub struct ClosureCondition<M = ()>(pub Box<dyn Fn(&M) -> bool>);

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

impl<M> std::fmt::Debug for ClosureCondition<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureCondition").finish()
    }
}

/// Condition that wraps another condition and inverts/negates its result.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// let mut memory = Memory { counter: 1 };
/// let condition = ConditionInvert::new(
///     ClosureCondition::new(|memory: &Memory| memory.counter == 0),
/// );
/// assert!(condition.validate(&memory));
/// ```
pub struct ConditionInvert<M = ()>(pub Box<dyn Condition<M>>);

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

impl<M> std::fmt::Debug for ConditionInvert<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionInvert").finish()
    }
}

/// Condition that wraps [`Consideration`] and converts it into condition.
///
/// Returns true if consideration returns score greater than [`ConsiderationCondition::threshold`]
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { value: Scalar }
///
/// let mut memory = Memory { value: 1.0 };
/// let condition = ConsiderationCondition::new(
///     ClosureConsideration::new(|memory: &Memory| memory.value),
///     0.5,
/// );
/// assert!(condition.validate(&memory));
/// ```
pub struct ConsiderationCondition<M = ()> {
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

impl<M> std::fmt::Debug for ConsiderationCondition<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsiderationCondition")
            .field("threshold", &self.threshold)
            .finish()
    }
}
