//! Reasoner (a.k.a. Utility) decision maker.

use crate::{DefaultKey, Scalar, consideration::*, decision_makers::*, task::*};
use std::{collections::HashMap, hash::Hash};

/// Reasoner error.
pub enum ReasonerError<K = DefaultKey> {
    /// There is no state with given ID found in reasoner.
    StateDoesNotExists(K),
}

impl<K> Clone for ReasonerError<K>
where
    K: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::StateDoesNotExists(key) => Self::StateDoesNotExists(key.clone()),
        }
    }
}

impl<K> PartialEq for ReasonerError<K>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StateDoesNotExists(a), Self::StateDoesNotExists(b)) => a == b,
        }
    }
}

impl<K> Eq for ReasonerError<K> where K: Eq {}

impl<K> std::fmt::Debug for ReasonerError<K>
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

/// Defines machinery state with task to run and consideration to score.
pub struct ReasonerState<M = ()> {
    consideration: Box<dyn Consideration<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> ReasonerState<M> {
    /// Constructs new state with task and consideration.
    pub fn new<C, T>(consideration: C, task: T) -> Self
    where
        C: Consideration<M> + 'static,
        T: Task<M> + 'static,
    {
        Self {
            consideration: Box::new(consideration),
            task: Box::new(task),
        }
    }

    /// Constructs new state with task and consideration.
    pub fn new_raw(consideration: Box<dyn Consideration<M>>, task: Box<dyn Task<M>>) -> Self {
        Self {
            consideration,
            task,
        }
    }
}

impl<M> std::fmt::Debug for ReasonerState<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasonerState").finish()
    }
}

/// State selector for reasoner.
///
/// Defines how reasoner will pick state from scored states.
pub trait ReasonerStateSelector<M, K>: Send + Sync
where
    K: Clone + Hash + Eq,
{
    /// Selects state from scored states.
    fn select_state(&self, memory: &M, scored_states: &[(&K, Scalar)]) -> Option<K>;
}

/// Selects state with maximum score.
pub struct MaxReasonerStateSelector;

impl<M, K> ReasonerStateSelector<M, K> for MaxReasonerStateSelector
where
    K: Clone + Hash + Eq,
{
    fn select_state(&self, _memory: &M, scored_states: &[(&K, Scalar)]) -> Option<K> {
        scored_states
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(k, _)| (*k).clone())
    }
}

/// Selects state with minimum score.
pub struct MinReasonerStateSelector;

impl<M, K> ReasonerStateSelector<M, K> for MinReasonerStateSelector
where
    K: Clone + Hash + Eq,
{
    fn select_state(&self, _memory: &M, scored_states: &[(&K, Scalar)]) -> Option<K> {
        scored_states
            .iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(k, _)| (*k).clone())
    }
}

/// Selects state with score closest to given value.
pub struct ClosestToReasonerStateSelector(pub Scalar);

impl<M, K> ReasonerStateSelector<M, K> for ClosestToReasonerStateSelector
where
    K: Clone + Hash + Eq,
{
    fn select_state(&self, _memory: &M, scored_states: &[(&K, Scalar)]) -> Option<K> {
        scored_states
            .iter()
            .min_by(|(_, a), (_, b)| (a - self.0).abs().partial_cmp(&(b - self.0).abs()).unwrap())
            .map(|(k, _)| (*k).clone())
    }
}

/// Reasoner (a.k.a. Utility AI).
///
/// Reasoner contains list of states with considerations that will score probability of given state
/// to happen. When states get scored, then it picks one with the highest score and change into it.
///
/// # Example
/// ```
/// use emergent::prelude::*;
/// use std::hash::Hash;
///
/// #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// enum Mode {
///     Low,
///     High,
/// }
///
/// struct Distance(pub Scalar);
///
/// impl Consideration<Scalar> for Distance {
///     fn score(&self, memory: &Scalar) -> Scalar {
///         1.0 - (self.0 - *memory).abs().min(1.0)
///     }
/// }
///
/// let mut reasoner = ReasonerBuilder::default()
///     .state(Mode::Low, ReasonerState::new(Distance(0.0), NoTask::default()))
///     .state(Mode::High, ReasonerState::new(Distance(1.0), NoTask::default()))
///     .build();
///
/// let mut memory = 0.0;
/// assert!(reasoner.process(&mut memory));
/// assert_eq!(reasoner.active_state(), Some(&Mode::Low));
/// memory = 1.0;
/// assert!(reasoner.process(&mut memory));
/// assert_eq!(reasoner.active_state(), Some(&Mode::High));
/// ```
pub struct Reasoner<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    states: HashMap<K, ReasonerState<M>>,
    active_state: Option<K>,
    state_selector: Box<dyn ReasonerStateSelector<M, K>>,
}

impl<M, K> Reasoner<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Construct new reasoner with states.
    pub fn new(states: HashMap<K, ReasonerState<M>>) -> Self {
        Self::with_selector(states, MaxReasonerStateSelector)
    }

    /// Construct new reasoner with states and state selector.
    pub fn with_selector<SS>(states: HashMap<K, ReasonerState<M>>, state_selector: SS) -> Self
    where
        SS: ReasonerStateSelector<M, K> + 'static,
    {
        Self {
            states,
            active_state: None,
            state_selector: Box::new(state_selector),
        }
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
    ) -> Result<bool, ReasonerError<K>> {
        if id == self.active_state {
            return Ok(false);
        }
        if let Some(id) = &id
            && !self.states.contains_key(id)
        {
            return Err(ReasonerError::StateDoesNotExists(id.clone()));
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

    ///Performs decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        if self.states.is_empty() {
            return false;
        }
        let scored_ids = self
            .states
            .iter()
            .map(|(id, state)| (id, state.consideration.score(memory)))
            .collect::<Vec<_>>();
        let Some(new_id) = self.state_selector.select_state(memory, &scored_ids) else {
            return false;
        };
        if let Ok(true) = self.change_active_state(Some(new_id), memory, false) {
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

impl<M, K> DecisionMaker<M, K> for Reasoner<M, K>
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

impl<M, K> Task<M> for Reasoner<M, K>
where
    K: Clone + Hash + Eq + Send + Sync,
{
    fn is_locked(&self, memory: &M) -> bool {
        if let Some(id) = &self.active_state
            && let Some(state) = self.states.get(id)
        {
            return state.task.is_locked(memory);
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

impl<M, K> std::fmt::Debug for Reasoner<M, K>
where
    K: Clone + Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reasoner")
            .field("states", &self.states)
            .field("active_state", &self.active_state)
            .finish()
    }
}

/// Reasoner builder.
///
/// See [`Reasoner`].
pub struct ReasonerBuilder<M = (), K = DefaultKey>(pub HashMap<K, ReasonerState<M>>);

impl<M, K> Default for ReasonerBuilder<M, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M, K> ReasonerBuilder<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Add new state.
    pub fn state(mut self, id: K, state: ReasonerState<M>) -> Self {
        self.0.insert(id, state);
        self
    }

    /// Consume builder and build new reasoner.
    pub fn build(self) -> Reasoner<M, K>
    where
        K: Clone + Hash + Eq,
    {
        Reasoner::new(self.0)
    }

    /// Consume builder and build new reasoner with state selector.
    pub fn build_with_state_selector<SS>(self, state_selector: SS) -> Reasoner<M, K>
    where
        K: Clone + Hash + Eq,
        SS: ReasonerStateSelector<M, K> + 'static,
    {
        Reasoner::with_selector(self.0, state_selector)
    }
}

impl<M, K> std::fmt::Debug for ReasonerBuilder<M, K>
where
    K: Clone + Hash + Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasonerBuilder")
            .field("states", &self.0)
            .finish()
    }
}
