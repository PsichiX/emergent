//! Decision makers are engines that usually contain states and decide under what circumstances
//! switch into which state.
//!
//! Engines
//! ---
//! - [`Machinery`](struct@self::machinery::Machinery) - Finite State Machine (or simply network of
//!   states connected by conditions to met for jumps to happen).
//! - [`Reasoner`](struct@self::reasoner::Reasoner) - Utility AI agent (that scores each state and
//!   selects one with the highest score).
//! - [`Planner`](struct@self::planner::Planner) - Goal Oriented Action Planning agent (finds the
//!   best path through all possible actions for goal selected by another decision maker assigned
//!   into this planner).
//! - [`Sequencer`](struct@self::sequencer::Sequencer) - Goes through states (ones that are possible
//!   to run) in a sequence.
//! - [`Selector`](struct@self::selector::Selector) - Selects only one state from list of possible
//!   states to run.
//! - [`Parallelizer`](struct@self::parallelizer::Parallelizer) - Runs all states (that are possible
//!   to run) at the same time.
//!
//! Modularity and hierarchical composition
//! ---
//! The main goal of this crate is to provide a way to construct modern AI solutions by combining
//! smaller decision making engines.
//!
//! Let me show some examples to clarify how this modularity helps building more complex AI:
//!
//! HFSM
//! ---
//! See: [https://cps-vo.org/group/hfsm](https://cps-vo.org/group/hfsm)
//!
//! One common AI technique is __HFSM__ (Hierarchical Finite State Machine) used to optimize FSM
//! networks (number of connections) by grouping sub-networks into clusters of states and connect
//! these clusters. Imagine you have states such as: [Eat, Sleep, Work, Drive].
//!
//! Instead of connecting each one with every other states like this:
//! - Eat
//! - Sleep
//! - Work
//! - Drive
//!
//! you group them into hierarchy of two levels with and connect only states that are on the same
//! level of hierarchy. This produces two levels of hierarchy and reduces number of connections
//! between them:
//! - Home:
//!   - Eat
//!   - Sleep
//! - Workplace:
//!   - Eat
//!   - Work
//! - Drive
//!
//! Behavior Tree
//! ---
//! See: [https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control)](https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control))
//!
//! Another commonly used AI technique is __Behavior Tree__ that evaluates tree nodes from the top
//! left to the bottom right as long as nodes succeeds. To make behavior trees possible with this
//! crate, you can just combine [`Sequencer`](struct@self::sequencer::Sequencer),
//! [`Selector`](struct@self::selector::Selector) and [`Task`](crate::task::Task) manually in a tree,
//! or use [`BehaviorTree`](enum@crate::builders::behavior_tree::BehaviorTree) builder to easily
//! define a tree and let builder produce properly setup tree of decision makers:
//! - Selector:
//!   - Drive
//!   - Sequence (Home):
//!     - Sleep
//!     - Eat
//!   - Sequence (Workplace):
//!     - Work
//!     - Eat

pub mod machinery;
pub mod parallelizer;
pub mod planner;
pub mod reasoner;
pub mod selector;
pub mod sequencer;

use crate::DefaultKey;
use std::marker::PhantomData;

/// Iterface for all decision making engines.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Switcher<M = ()> {
///     states: [Box<dyn Task<M>>; 2],
///     active_index: Option<usize>,
/// }
///
/// impl<M> DecisionMaker<M, usize> for Switcher<M> {
///     fn decide(&mut self, memory: &mut M) -> Option<usize> {
///         if let Some(index) = self.active_index {
///             self.states[index].on_exit(memory);
///         }
///         let index = self.active_index.map(|index| (index + 1) % 2).unwrap_or_default();
///         self.states[index].on_enter(memory);
///         self.active_index = Some(index);
///         self.active_index
///     }
///
///     fn change_mind(&mut self, id: Option<usize>, memory: &mut M) -> bool {
///         if id == self.active_index {
///             return false;
///         }
///         if let Some(index) = self.active_index {
///             self.states[index].on_exit(memory);
///         }
///         if let Some(index) = id {
///             self.states[index].on_enter(memory);
///         }
///         self.active_index = id;
///         true
///     }
/// }
///
/// let mut switcher = Switcher {
///     states: [
///         Box::new(NoTask::default()),
///         Box::new(NoTask::default()),
///     ],
///     active_index: None,
/// };
///
/// assert_eq!(switcher.active_index, None);
/// assert_eq!(switcher.decide(&mut ()), Some(0));
/// assert_eq!(switcher.decide(&mut ()), Some(1));
/// assert_eq!(switcher.decide(&mut ()), Some(0));
/// assert!(switcher.change_mind(None, &mut ()));
/// assert_eq!(switcher.active_index, None);
/// ```
pub trait DecisionMaker<M = (), K = DefaultKey>: Send + Sync {
    /// Performs decision making and returns the key of the state it switched into.
    fn decide(&mut self, memory: &mut M) -> Option<K>;

    /// Force switch into state (`Some` activates new state, `None` deactivates current state).
    ///
    /// Returns true if successfully changed into new state.
    fn change_mind(&mut self, id: Option<K>, memory: &mut M) -> bool;
}

/// Empty decision maker that simply does nothing.
#[allow(clippy::type_complexity)]
pub struct NoDecisionMaker<M = (), K = DefaultKey>(PhantomData<(fn() -> M, fn() -> K)>);

impl<M, K> Default for NoDecisionMaker<M, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M, K> DecisionMaker<M, K> for NoDecisionMaker<M, K> {
    fn decide(&mut self, _: &mut M) -> Option<K> {
        None
    }

    fn change_mind(&mut self, _: Option<K>, _: &mut M) -> bool {
        false
    }
}

impl<M, K> std::fmt::Debug for NoDecisionMaker<M, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoDecisionMaker").finish()
    }
}

/// Single choice decision maker (it always takes single specified decision).
pub struct SingleDecisionMaker<K = DefaultKey> {
    id: K,
    active: bool,
}

impl<K> SingleDecisionMaker<K>
where
    K: Clone + Eq,
{
    /// Constructs new single choice decision maker.
    pub fn new(id: K) -> Self {
        Self { id, active: false }
    }

    /// Returns active state ID.
    pub fn active_state(&self) -> Option<&K> {
        if self.active {
            Some(&self.id)
        } else {
            None
        }
    }
}

impl<M, K> DecisionMaker<M, K> for SingleDecisionMaker<K>
where
    K: Clone + Eq + Send + Sync,
{
    fn decide(&mut self, _: &mut M) -> Option<K> {
        self.active = true;
        Some(self.id.clone())
    }

    fn change_mind(&mut self, id: Option<K>, _: &mut M) -> bool {
        if let Some(id) = id {
            if !self.active && self.id == id {
                self.active = true;
                return true;
            }
        } else if self.active {
            self.active = false;
            return true;
        }
        false
    }
}

impl<K> std::fmt::Debug for SingleDecisionMaker<K>
where
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleDecisionMaker")
            .field("id", &self.id)
            .field("active", &self.active)
            .finish()
    }
}
