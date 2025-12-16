//! Runs first state that succeeds (boolean OR operation).

use crate::{condition::*, decision_makers::*, task::*};

/// Defines selector state with task and condition.
pub struct SelectorState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> SelectorState<M> {
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

impl<M> std::fmt::Debug for SelectorState<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectorState").finish()
    }
}

/// Selector runs at most only one state at any given time.
///
/// Selector finds first state that its condition succeed and runs it.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Is(pub bool);
///
/// impl Condition<bool> for Is {
///     fn validate(&self, memory: &bool) -> bool {
///         *memory == self.0
///     }
/// }
///
/// struct Set(pub bool);
///
/// impl Task<bool> for Set {
///     fn on_enter(&mut self, memory: &mut bool) {
///         *memory = self.0
///     }
/// }
///
/// let mut selector = Selector::new(vec![
///     SelectorState::new(Is(true), Set(false)),
///     SelectorState::new(Is(false), Set(true)),
/// ]);
/// let mut memory = false;
/// assert!(selector.process(&mut memory));
/// assert_eq!(memory, true);
/// assert!(selector.process(&mut memory));
/// assert_eq!(memory, false);
/// ```
pub struct Selector<M = ()> {
    states: Vec<SelectorState<M>>,
    active_index: Option<usize>,
}

impl<M> Selector<M> {
    /// Constructs new selector from states.
    pub fn new(states: Vec<SelectorState<M>>) -> Self {
        Self {
            states,
            active_index: None,
        }
    }

    /// Constructs new selector from states.
    pub fn new_raw(states: Vec<SelectorState<M>>) -> Self {
        Self {
            states,
            active_index: None,
        }
    }

    /// Returns currently active state index.
    pub fn active_index(&self) -> Option<usize> {
        self.active_index
    }

    /// Reset currently active state.
    ///
    /// By default state won't change if active state is locked, but we can force state change.
    pub fn reset(&mut self, memory: &mut M, forced: bool) -> bool {
        if let Some(index) = self.active_index {
            let state = self.states.get_mut(index).unwrap();
            if !forced && state.task.is_locked(memory) {
                return false;
            }
            state.task.on_exit(memory);
            self.active_index = None;
        }
        true
    }

    fn change_active_index(&mut self, index: Option<usize>, memory: &mut M) -> bool {
        if index == self.active_index {
            return false;
        }
        if let Some(index) = self.active_index {
            let state = self.states.get_mut(index).unwrap();
            if state.task.is_locked(memory) {
                return false;
            }
            state.task.on_exit(memory);
        }
        if let Some(index) = index {
            self.states.get_mut(index).unwrap().task.on_enter(memory);
        }
        self.active_index = index;
        true
    }

    /// Perform decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        if self.states.is_empty() {
            return false;
        }
        let index = self
            .states
            .iter()
            .position(|state| state.condition.validate(memory));
        if self.change_active_index(index, memory) {
            return true;
        }
        if let Some(index) = self.active_index {
            return self.states.get_mut(index).unwrap().task.on_process(memory);
        }
        false
    }

    /// Update currently active state.
    pub fn update(&mut self, memory: &mut M) {
        if let Some(index) = self.active_index {
            self.states.get_mut(index).unwrap().task.on_update(memory);
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Selector<M>
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

impl<M> Task<M> for Selector<M> {
    fn is_locked(&self, memory: &M) -> bool {
        if let Some(index) = self.active_index
            && let Some(state) = self.states.get(index)
        {
            return state.task.is_locked(memory);
        }
        false
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

impl<M> std::fmt::Debug for Selector<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Selector")
            .field("states", &self.states)
            .field("active_index", &self.active_index)
            .finish()
    }
}
