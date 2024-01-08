//! Run states one-by-one as long as they succeed (boolean AND operation).

use crate::{condition::*, decision_makers::*, task::*};

/// Defines sequencer state with task and condition.
pub struct SequencerState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> SequencerState<M> {
    /// Constructs new state with condition and state.
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

    /// Constructs new state with condition and state.
    pub fn new_raw(condition: Box<dyn Condition<M>>, task: Box<dyn Task<M>>) -> Self {
        Self { condition, task }
    }
}

impl<M> std::fmt::Debug for SequencerState<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequencerState").finish()
    }
}

/// Sequencer runs states one by one.
///
/// Sequencer has two properties that change its behavior:
/// - Looping (see [`Self::is_looped`])
/// - Continuity (see [`Self::does_continue`])
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let mut sequencer = Sequencer::new(
///     vec![
///         SequencerState::new(true, NoTask::default()),
///         SequencerState::new(false, NoTask::default()),
///         SequencerState::new(true, NoTask::default()),
///         SequencerState::new(false, NoTask::default()),
///     ],
///     true,
///     true,
/// );
///
/// assert_eq!(sequencer.active_index(), None);
/// assert!(sequencer.process(&mut ()));
/// assert_eq!(sequencer.active_index(), Some(0));
/// assert!(sequencer.process(&mut ()));
/// assert_eq!(sequencer.active_index(), Some(2));
/// assert!(sequencer.process(&mut ()));
/// assert_eq!(sequencer.active_index(), Some(0));
/// ```
pub struct Sequencer<M = ()> {
    states: Vec<SequencerState<M>>,
    active_index: Option<usize>,
    looped: bool,
    continuity: bool,
}

impl<M> Sequencer<M> {
    /// Constructs new sequencer with states, looping and continuity settings.
    ///
    /// See [`Self::is_looped`].
    /// See [`Self::does_continue`].
    pub fn new(states: Vec<SequencerState<M>>, looped: bool, continuity: bool) -> Self {
        Self {
            states,
            active_index: None,
            looped,
            continuity,
        }
    }

    /// Returns currently active state index.
    pub fn active_index(&self) -> Option<usize> {
        self.active_index
    }

    /// Tells if when reaches end of the sequence, starts back from first state.
    pub fn is_looped(&self) -> bool {
        self.looped
    }

    /// Tells if sequence cannot break.
    ///
    /// When continuity is false, it means whenever tries to find next state to change, that state
    /// has to succeed or else whole sequence stops.
    pub fn does_continue(&self) -> bool {
        self.continuity
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
        let index = if let Some(index) = self.active_index {
            if self.looped {
                if self.continuity {
                    self.states
                        .iter()
                        .enumerate()
                        .cycle()
                        .skip(index + 1)
                        .take(self.states.len())
                        .find_map(|(index, state)| {
                            if state.condition.validate(memory) {
                                Some(index)
                            } else {
                                None
                            }
                        })
                } else {
                    match self.states.iter().enumerate().cycle().nth(index + 1) {
                        Some((index, state)) => {
                            if state.condition.validate(memory) {
                                Some(index)
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                }
            } else if self.continuity {
                self.states
                    .iter()
                    .enumerate()
                    .skip(index + 1)
                    .find_map(|(index, state)| {
                        if state.condition.validate(memory) {
                            Some(index)
                        } else {
                            None
                        }
                    })
            } else {
                match self.states.iter().enumerate().nth(index + 1) {
                    Some((index, state)) => {
                        if state.condition.validate(memory) {
                            Some(index)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            }
        } else {
            self.states
                .iter()
                .position(|state| state.condition.validate(memory))
        };
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

impl<M, K> DecisionMaker<M, K> for Sequencer<M>
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

impl<M> Task<M> for Sequencer<M> {
    fn is_locked(&self, memory: &M) -> bool {
        if let Some(index) = self.active_index {
            if let Some(state) = self.states.get(index) {
                return state.task.is_locked(memory);
            }
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

impl<M> std::fmt::Debug for Sequencer<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sequencer")
            .field("states", &self.states)
            .field("active_index", &self.active_index)
            .field("looped", &self.looped)
            .field("continuity", &self.continuity)
            .finish()
    }
}
