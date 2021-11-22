//! Runs one state that succeeds (boolean OR operation).

use crate::{condition::*, decision_makers::*, task::*, DefaultKey};
use std::{collections::HashMap, hash::Hash};

/// Selector error.
pub enum SelectorError<K = DefaultKey> {
    /// Selector doesn't have state with given ID.
    StateDoesNotExists(K),
}

impl<K> Clone for SelectorError<K>
where
    K: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::StateDoesNotExists(key) => Self::StateDoesNotExists(key.clone()),
        }
    }
}

impl<K> PartialEq for SelectorError<K>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StateDoesNotExists(a), Self::StateDoesNotExists(b)) => a == b,
        }
    }
}

impl<K> Eq for SelectorError<K> where K: Eq {}

impl<K> std::fmt::Debug for SelectorError<K>
where
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateDoesNotExists(key) => {
                write!(f, "StateDoesNotExists({:?})", key)
            }
        }
    }
}

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
/// First selector finds all states that their condition succeed, then uses state picker to decide
/// what state to choose from the list of successful states.
///
/// # Example
/// ```
/// use emergent::prelude::*;
/// use std::{collections::HashMap, hash::Hash};
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
/// let mut states = HashMap::new();
/// states.insert(false, SelectorState::new(Is(true), Set(false)));
/// states.insert(true, SelectorState::new(Is(false), Set(true)));
///
/// let mut selector = Selector::new((), states);
/// let mut memory = false;
/// assert!(selector.process(&mut memory));
/// assert_eq!(memory, true);
/// assert!(selector.process(&mut memory));
/// assert_eq!(memory, false);
/// ```
pub struct Selector<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    state_picker: Box<dyn SelectorStatePicker<M, K>>,
    states: HashMap<K, SelectorState<M>>,
    active_state: Option<K>,
}

impl<M, K> Selector<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Constructs new selector with state picker and states.
    pub fn new<P>(state_picker: P, states: HashMap<K, SelectorState<M>>) -> Self
    where
        P: SelectorStatePicker<M, K> + 'static,
    {
        Self {
            state_picker: Box::new(state_picker),
            states,
            active_state: None,
        }
    }

    /// Constructs new selector with state picker and states.
    pub fn new_raw(
        state_picker: Box<dyn SelectorStatePicker<M, K>>,
        states: HashMap<K, SelectorState<M>>,
    ) -> Self {
        Self {
            state_picker,
            states,
            active_state: None,
        }
    }

    /// Returns currently active state ID.
    pub fn active_state(&self) -> Option<&K> {
        self.active_state.as_ref()
    }

    /// Change currently active state.
    ///
    /// By default state won't change if active state is locked, but we can force state change.
    pub fn change_active_state(
        &mut self,
        id: Option<K>,
        memory: &mut M,
        forced: bool,
    ) -> Result<bool, SelectorError<K>> {
        if id == self.active_state {
            return Ok(false);
        }
        if let Some(id) = &id {
            if !self.states.contains_key(id) {
                return Err(SelectorError::StateDoesNotExists(id.clone()));
            }
        }
        if let Some(id) = &self.active_state {
            let state = self.states.get_mut(id).unwrap();
            if !forced && state.task.is_locked(memory) {
                return Ok(false);
            }
            state.task.on_exit(memory);
        }
        if let Some(id) = &id {
            self.states.get_mut(id).unwrap().task.on_enter(memory);
        }
        self.active_state = id;
        Ok(true)
    }

    /// Perform decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        let available = self
            .states
            .iter()
            .filter(|(_, state)| state.condition.validate(memory))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        let new_id = if available.is_empty() {
            None
        } else {
            self.state_picker.pick(&available, memory)
        };
        if let Ok(true) = self.change_active_state(new_id, memory, false) {
            return true;
        }
        if let Some(id) = &self.active_state {
            return self.states.get_mut(id).unwrap().task.on_process(memory);
        }
        false
    }

    /// Update currently active state.
    pub fn update(&mut self, memory: &mut M) {
        if let Some(id) = &self.active_state {
            self.states.get_mut(id).unwrap().task.on_update(memory);
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Selector<M, K>
where
    K: Clone + Hash + Eq + Send + Sync,
{
    fn decide(&mut self, memory: &mut M) -> Option<K> {
        self.process(memory);
        self.active_state().cloned()
    }

    fn change_mind(&mut self, id: Option<K>, memory: &mut M) -> bool {
        matches!(self.change_active_state(id, memory, true), Ok(true))
    }
}

impl<M, K> Task<M> for Selector<M, K>
where
    K: Clone + Hash + Eq + Send + Sync,
{
    fn is_locked(&self, memory: &M) -> bool {
        if let Some(id) = &self.active_state {
            if let Some(state) = self.states.get(id) {
                return state.task.is_locked(memory);
            }
        }
        false
    }

    fn on_enter(&mut self, memory: &mut M) {
        let _ = self.change_active_state(None, memory, true);
        self.process(memory);
    }

    fn on_exit(&mut self, memory: &mut M) {
        let _ = self.change_active_state(None, memory, true);
    }

    fn on_update(&mut self, memory: &mut M) {
        self.update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.process(memory)
    }
}

impl<M, K> std::fmt::Debug for Selector<M, K>
where
    K: Clone + Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Selector")
            .field("states", &self.states)
            .field("active_state", &self.active_state)
            .finish()
    }
}

/// Picks single state ID from array of successful states.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct PickMiddleOne;
///
/// impl<M, K> SelectorStatePicker<M, K> for PickMiddleOne where K: Clone {
///     fn pick(&mut self, available: &[&K], memory: &M) -> Option<K> {
///         available.iter().nth(available.len() / 2).map(|id| (**id).clone())
///     }
/// }
///
/// let states = vec![&"a", &"b", &"c"];
/// assert_eq!(PickMiddleOne.pick(&states, &()), Some("b"));
/// ```
pub trait SelectorStatePicker<M = (), K = DefaultKey>: Send + Sync {
    /// Pick some or none state that wins.
    fn pick(&mut self, available: &[&K], memory: &M) -> Option<K>;
}

impl<M, K> SelectorStatePicker<M, K> for ()
where
    K: Clone,
{
    fn pick(&mut self, available: &[&K], _: &M) -> Option<K> {
        available.first().map(|id| (*id).clone())
    }
}

/// Wraps closure in selector state picker.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let mut picker = ClosureSelectorStatePicker::<(), &str>::new(
///     |ids, _| ids.first().map(|id| (**id).clone()),
/// );
/// let states = vec![&"a", &"b", &"c"];
/// assert_eq!(picker.pick(&states, &()), Some("a"));
/// ```
pub struct ClosureSelectorStatePicker<M = (), K = DefaultKey>(
    Box<dyn FnMut(&[&K], &M) -> Option<K> + Send + Sync>,
);

impl<M, K> ClosureSelectorStatePicker<M, K> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&[&K], &M) -> Option<K> + 'static + Send + Sync,
    {
        Self(Box::new(f))
    }
}

impl<M, K> SelectorStatePicker<M, K> for ClosureSelectorStatePicker<M, K> {
    fn pick(&mut self, available: &[&K], memory: &M) -> Option<K> {
        (self.0)(available, memory)
    }
}

/// Selects first, last or nth available state ID.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// let states = vec![&"a", &"b", &"c"];
/// assert_eq!(OrderedSelectorStatePicker::Last.pick(&states, &()), Some("c"));
/// ```
pub enum OrderedSelectorStatePicker {
    First,
    Last,
    Nth(usize),
}

impl<M, K> SelectorStatePicker<M, K> for OrderedSelectorStatePicker
where
    K: Clone,
{
    fn pick(&mut self, available: &[&K], _: &M) -> Option<K> {
        match self {
            Self::First => available.first().map(|id| (**id).clone()),
            Self::Last => available.last().map(|id| (**id).clone()),
            Self::Nth(index) => available.get(*index).map(|id| (**id).clone()),
        }
    }
}
