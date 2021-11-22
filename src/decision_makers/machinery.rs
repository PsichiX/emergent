//! Machinery (a.k.a. Finite State Machine) decision maker.

use crate::{condition::*, decision_makers::*, task::*, DefaultKey};
use std::{collections::HashMap, hash::Hash};

/// Machinery error.
pub enum MachineryError<K = DefaultKey> {
    /// There is no state with given ID found in machinery.
    StateDoesNotExists(K),
}

impl<K> Clone for MachineryError<K>
where
    K: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::StateDoesNotExists(key) => Self::StateDoesNotExists(key.clone()),
        }
    }
}

impl<K> PartialEq for MachineryError<K>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StateDoesNotExists(a), Self::StateDoesNotExists(b)) => a == b,
        }
    }
}

impl<K> Eq for MachineryError<K> where K: Eq {}

impl<K> std::fmt::Debug for MachineryError<K>
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

/// Defines a change to another state.
pub struct MachineryChange<M = (), K = DefaultKey> {
    /// Target state ID.
    pub to: K,
    /// Condition to met for change to happen.
    pub condition: Box<dyn Condition<M>>,
}

impl<M, K> MachineryChange<M, K> {
    /// Constructs new change descriptor with ID and condition.
    pub fn new<C>(to: K, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self {
            to,
            condition: Box::new(condition),
        }
    }

    /// Constructs new change descriptor with ID and condition.
    pub fn new_raw(to: K, condition: Box<dyn Condition<M>>) -> Self {
        Self { to, condition }
    }

    /// Test this change condition.
    pub fn validate(&self, memory: &M) -> bool {
        self.condition.validate(memory)
    }
}

impl<M, K> std::fmt::Debug for MachineryChange<M, K>
where
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MachineryChange")
            .field("to", &self.to)
            .finish()
    }
}

/// Defines machinery state with task to run and changes that can happen for this state.
pub struct MachineryState<M = (), K = DefaultKey> {
    task: Box<dyn Task<M>>,
    changes: Vec<MachineryChange<M, K>>,
}

impl<M, K> MachineryState<M, K> {
    /// Construct new state with task only.
    pub fn task<T>(task: T) -> Self
    where
        T: Task<M> + 'static,
    {
        Self {
            task: Box::new(task),
            changes: vec![],
        }
    }

    /// Construct new state with task only.
    pub fn task_raw(task: Box<dyn Task<M>>) -> Self {
        Self {
            task,
            changes: vec![],
        }
    }

    /// Constructs new state with task and list of changes.
    pub fn new<T>(task: T, changes: Vec<MachineryChange<M, K>>) -> Self
    where
        T: Task<M> + 'static,
    {
        Self {
            task: Box::new(task),
            changes,
        }
    }

    /// Constructs new state with task and list of changes.
    pub fn new_raw(task: Box<dyn Task<M>>, changes: Vec<MachineryChange<M, K>>) -> Self {
        Self { task, changes }
    }

    /// Add state change.
    pub fn change(mut self, change: MachineryChange<M, K>) -> Self {
        self.changes.push(change);
        self
    }
}

impl<M, K> std::fmt::Debug for MachineryState<M, K>
where
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MachineryState")
            .field("changes", &self.changes)
            .finish()
    }
}

/// Machinery (a.k.a. Finite State Machine).
///
/// Finite state machines are sets of states with possible transitions between them. Each transition
/// contains ID of another state to change into and condition to succeed for that transition to happen.
/// Yes, that's all - FSM are the simplest AI techique of them all.
///
/// How it works
/// ---
///
/// Imagine we define FSM like this:
/// - Do nothing:
///   - Eat (is hungry?)
///   - Sleep (low energy?)
/// - Eat:
///   - Do nothing (always succeed)
/// - Sleep:
///   - Do nothing (always succeed)
///
/// And we start with initial memory state:
/// - hungry: false
/// - low energy: false
///
/// We start at __doing nothing__ and we test all its transitions, but since agent neither is hungry
/// nor has low energy no change occur.
///
/// Let's change something in the state:
/// - __hungry: true__
/// - __low energy: true__
///
/// Now we test possible transitions again and we find that agent is both hungry and has low energy,
/// but since eat state succeeds first, agent is gonna __eat__ something. From that state the only
/// possible change is to do nothing again since it always succeeds.
///
/// Although this is a really small network of states and changes, usually the more states it gets,
/// the more connections and at some point we end up with very messy networks and at that point we
/// should switch either to Hierarchical State Machines or to other decision makers that are designed
/// to reduce number of changes.
///
/// See [https://en.wikipedia.org/wiki/Finite-state_machine](https://en.wikipedia.org/wiki/Finite-state_machine)
///
/// # Example
/// ```
/// use emergent::prelude::*;
/// use std::hash::Hash;
///
/// #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// enum Action {
///     None,
///     Eat,
///     Sleep,
/// }
///
/// struct IsAction(pub Action);
///
/// impl Condition<Action> for IsAction {
///     fn validate(&self, memory: &Action) -> bool {
///         *memory == self.0
///     }
/// }
///
/// let mut machinery = MachineryBuilder::default()
///     .state(
///         Action::None,
///         MachineryState::task(NoTask::default())
///             .change(MachineryChange::new(Action::Eat, IsAction(Action::Eat)))
///             .change(MachineryChange::new(Action::Sleep, IsAction(Action::Sleep))),
///     )
///     .state(
///         Action::Eat,
///         MachineryState::task(NoTask::default())
///             .change(MachineryChange::new(Action::None, true)),
///     )
///     .state(
///         Action::Sleep,
///         MachineryState::task(NoTask::default())
///             .change(MachineryChange::new(Action::None, true)),
///     )
///     .build();
///
/// let mut memory = Action::Eat;
/// machinery.change_active_state(Some(Action::None), &mut memory, true);
/// assert!(machinery.process(&mut memory));
/// assert_eq!(machinery.active_state(), Some(&Action::Eat));
/// assert!(machinery.process(&mut memory));
/// assert_eq!(machinery.active_state(), Some(&Action::None));
/// memory = Action::Sleep;
/// assert!(machinery.process(&mut memory));
/// assert_eq!(machinery.active_state(), Some(&Action::Sleep));
/// assert!(machinery.process(&mut memory));
/// assert_eq!(machinery.active_state(), Some(&Action::None));
/// ```
pub struct Machinery<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    states: HashMap<K, MachineryState<M, K>>,
    active_state: Option<K>,
    initial_state_decision_maker: Option<Box<dyn DecisionMaker<M, K>>>,
}

impl<M, K> Machinery<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Construct new machinery with states.
    pub fn new(states: HashMap<K, MachineryState<M, K>>) -> Self {
        Self {
            states,
            active_state: None,
            initial_state_decision_maker: None,
        }
    }

    /// Assigns decision maker that will set initial state when machinery gets activated.
    ///
    /// This is useful when we want to use machinery in hierarchy.
    pub fn initial_state_decision_maker<DM>(mut self, decision_maker: DM) -> Self
    where
        DM: DecisionMaker<M, K> + 'static,
    {
        self.initial_state_decision_maker = Some(Box::new(decision_maker));
        self
    }

    /// Assigns decision maker that will set initial state when machinery gets activated.
    ///
    /// This is useful when we want to use machinery in hierarchy.
    pub fn initial_state_decision_maker_raw(
        mut self,
        decision_maker: Box<dyn DecisionMaker<M, K>>,
    ) -> Self {
        self.initial_state_decision_maker = Some(decision_maker);
        self
    }

    /// Returns currently active state ID.
    pub fn active_state(&self) -> Option<&K> {
        self.active_state.as_ref()
    }

    /// Change active state.
    ///
    /// If currently active state is locked then state change will fail, unless we force it to change.
    pub fn change_active_state(
        &mut self,
        id: Option<K>,
        memory: &mut M,
        forced: bool,
    ) -> Result<bool, MachineryError<K>> {
        if id == self.active_state {
            return Ok(false);
        }
        if let Some(id) = &id {
            if !self.states.contains_key(id) {
                return Err(MachineryError::StateDoesNotExists(id.clone()));
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

    /// Performs decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        if let Some(id) = &self.active_state {
            if let Some(state) = self.states.get_mut(id) {
                let id = state
                    .changes
                    .iter()
                    .find_map(|c| {
                        if c.validate(memory) {
                            Some(&c.to)
                        } else {
                            None
                        }
                    })
                    .cloned();
                if let Some(id) = id {
                    if let Ok(true) = self.change_active_state(Some(id), memory, false) {
                        return true;
                    }
                }
            }
        }
        if let Some(id) = &self.active_state {
            return self.states.get_mut(id).unwrap().task.on_process(memory);
        }
        false
    }

    /// Updates active state.
    pub fn update(&mut self, memory: &mut M) {
        if let Some(id) = &self.active_state {
            self.states.get_mut(id).unwrap().task.on_update(memory);
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Machinery<M, K>
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

impl<M, K> Task<M> for Machinery<M, K>
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
        let id = match &mut self.initial_state_decision_maker {
            Some(decision_maker) => decision_maker.decide(memory),
            None => None,
        };
        let _ = self.change_active_state(id, memory, true);
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

impl<M, K> std::fmt::Debug for Machinery<M, K>
where
    K: Clone + Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Machinery")
            .field("states", &self.states)
            .field("active_state", &self.active_state)
            .finish()
    }
}

/// Machinery builder.
///
/// See [`Machinery`].
pub struct MachineryBuilder<M = (), K = DefaultKey>(pub HashMap<K, MachineryState<M, K>>);

impl<M, K> Default for MachineryBuilder<M, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M, K> MachineryBuilder<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Add new state.
    pub fn state(mut self, id: K, state: MachineryState<M, K>) -> Self {
        self.0.insert(id, state);
        self
    }

    /// Consume builder and build new machinery.
    pub fn build(self) -> Machinery<M, K>
    where
        K: Clone + Hash + Eq,
    {
        Machinery::new(self.0)
    }
}

impl<M, K> std::fmt::Debug for MachineryBuilder<M, K>
where
    K: Clone + Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MachineryBuilder")
            .field("states", &self.0)
            .finish()
    }
}
