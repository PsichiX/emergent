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

pub mod builders;
pub mod combinators;
pub mod condition;
pub mod consideration;
pub mod decision_makers;
pub mod evaluators;
pub mod score_mapping;
pub mod task;

#[cfg(test)]
pub mod tests;

#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;
#[cfg(feature = "scalar64")]
pub type Scalar = f64;

pub type DefaultKey = String;

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
        score_mapping::*,
        task::*,
        DefaultKey, Scalar,
    };
}
