//! Run states in parallel.

use crate::{condition::*, decision_makers::*, task::*};

/// Defines parallelizer state with condition to succeed and task to run.
pub struct ParallelizerState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> ParallelizerState<M> {
    /// Constructs new state with condition and task.
    pub fn new<C, T>(condition: C, task: T) -> Self
    where
        C: Condition<M> + 'static,
        T: Task<M> + 'static,
    {
        Self {
            condition: Box::new(condition),
            task: Box::new(task),
        }
    }

    /// Constructs new state with condition and task.
    pub fn new_raw(condition: Box<dyn Condition<M>>, task: Box<dyn Task<M>>) -> Self {
        Self { condition, task }
    }
}

impl<M> std::fmt::Debug for ParallelizerState<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelizerState").finish()
    }
}

/// Parallelizer runs all its states at the same time.
///
/// Note that at any time you ask it to make a decision it goes through non-active states and tries
/// to run them so instead of starting all possible states at once, it will ensure that at any time
/// of decision making all possible states will run.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory {
///     a: bool,
///     b: bool,
/// }
///
/// let mut parallelizer = Parallelizer::new(vec![
///     ParallelizerState::new(true, ClosureTask::default().enter(|m: &mut Memory| m.a = true)),
///     ParallelizerState::new(true, ClosureTask::default().enter(|m: &mut Memory| m.b = true)),
/// ]);
///
/// let mut memory = Memory { a: false, b: false };
/// assert!(parallelizer.process(&mut memory));
/// assert_eq!(memory.a, true);
/// assert_eq!(memory.b, true);
/// ```
pub struct Parallelizer<M = ()> {
    states: Vec<(ParallelizerState<M>, bool)>,
}

impl<M> Parallelizer<M> {
    /// Constructs new parallelizer with states.
    pub fn new(states: Vec<ParallelizerState<M>>) -> Self {
        Self {
            states: states.into_iter().map(|state| (state, false)).collect(),
        }
    }

    /// Tells if any of states is active/running.
    pub fn is_active(&self) -> bool {
        self.states.iter().any(|(_, active)| *active)
    }

    /// Stops all active/running states.
    ///
    /// By default states that are locked won't stop, but we can force stop them.
    pub fn reset(&mut self, memory: &mut M, forced: bool) -> bool {
        let mut result = false;
        for (state, active) in &mut self.states {
            if *active && (forced || !state.task.is_locked(memory)) {
                state.task.on_exit(memory);
                *active = false;
                result = true;
            }
        }
        result
    }

    /// Perform decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        let mut result = false;
        for (state, active) in &mut self.states {
            if *active {
                if state.task.is_locked(memory) && state.condition.validate(memory) {
                    if state.task.on_process(memory) {
                        result = true;
                    }
                } else {
                    state.task.on_exit(memory);
                    *active = false;
                    result = true;
                }
            } else if state.condition.validate(memory) {
                state.task.on_enter(memory);
                *active = true;
                result = true;
            }
        }
        result
    }

    /// Update active/running states.
    pub fn update(&mut self, memory: &mut M) {
        for (state, active) in &mut self.states {
            if *active {
                state.task.on_update(memory);
            }
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Parallelizer<M>
where
    K: Default,
{
    fn decide(&mut self, memory: &mut M) -> Option<K> {
        self.process(memory);
        Some(K::default())
    }

    fn change_mind(&mut self, _: Option<K>, memory: &mut M) -> bool {
        self.reset(memory, true)
    }
}

impl<M> Task<M> for Parallelizer<M> {
    fn is_locked(&self, memory: &M) -> bool {
        self.states
            .iter()
            .any(|(state, active)| *active && state.task.is_locked(memory))
    }

    fn on_enter(&mut self, memory: &mut M) {
        self.reset(memory, true);
        self.process(memory);
    }

    fn on_exit(&mut self, memory: &mut M) {
        self.reset(memory, true);
    }

    fn on_update(&mut self, memory: &mut M) {
        self.update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.process(memory)
    }
}

impl<M> std::fmt::Debug for Parallelizer<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parallelizer")
            .field("states", &self.states)
            .finish()
    }
}
