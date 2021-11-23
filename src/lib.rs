//! __Modern, modular and hierarchical way to define complex behaviors using simple building blocks.__
//!
//! Main concepts
//! ---
//! - __Task__ - Units of work performed in time ([`crate::task`])
//! - __Decision Makers__ - State change engines ([`crate::decision_makers`])
//! - __Condition__- Answers to questions about certain memory states ([`crate::condition`]. [`crate::combinators`])
//! - __Considerations__ - Scored probabilities of certain memory states ([`crate::consideration`], [`crate::evaluators`])
//! - __Memory__ - Memory is the state passed to all concepts listed above to be read/write by them.
//!   In other AI systems memory is also known as blackboard or context.
//!
//! Note that AI techniques provided by this library are usable not only for agent behaviors but
//! also for building emergent storytelling when used for smart world events generation. In fact AI
//! can be used for automation and modularization of many aspects of game logic, not only agents
//! and events - consider your creativity being the only limit of what AI techniques can be used for.

pub mod builders;
pub mod combinators;
pub mod condition;
pub mod consideration;
pub mod decision_makers;
pub mod evaluators;
pub mod memory;
pub mod score_mapping;
pub mod task;

#[cfg(test)]
pub mod tests;

use crate::{decision_makers::DecisionMaker, task::Task};

#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;
#[cfg(feature = "scalar64")]
pub type Scalar = f64;

pub type DefaultKey = String;

pub trait DecisionMakingTask<M = (), K = DefaultKey>:
    DecisionMaker<M, K> + Task<M> + Sized
{
    fn as_decision_maker(&self) -> &dyn DecisionMaker<M, K> {
        self
    }

    fn as_decision_maker_mut(&mut self) -> &mut dyn DecisionMaker<M, K> {
        self
    }

    fn as_task(&self) -> &dyn Task<M> {
        self
    }

    fn as_task_mut(&mut self) -> &mut dyn Task<M> {
        self
    }

    fn into_decision_maker(self) -> Box<dyn DecisionMaker<M, K>>
    where
        Self: 'static,
    {
        Box::new(self)
    }

    fn into_task(self) -> Box<dyn Task<M>>
    where
        Self: 'static,
    {
        Box::new(self)
    }
}

impl<T, M, K> DecisionMakingTask<M, K> for T where T: DecisionMaker<M, K> + Task<M> + Sized {}

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        builders::{behavior_tree::*, lod::*, *},
        combinators::{all::*, any::*, count::*, *},
        condition::*,
        consideration::*,
        decision_makers::{
            machinery::*, parallelizer::*, planner::*, reasoner::*, selector::*, sequencer::*, *,
        },
        evaluators::{max::*, min::*, product::*, sum::*, *},
        memory::{blackboard::*, datatable::*, *},
        score_mapping::*,
        task::*,
        DecisionMakingTask, DefaultKey, Scalar,
    };
}
