use crate::{condition::*, decision_makers::*, task::*, DefaultKey};
use std::{collections::HashMap, hash::Hash};

pub enum SelectorError<K = DefaultKey> {
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

pub struct SelectorState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> SelectorState<M> {
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

    pub fn new_raw(condition: Box<dyn Condition<M>>, task: Box<dyn Task<M>>) -> Self {
        Self { condition, task }
    }
}

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

    pub fn active_state(&self) -> Option<&K> {
        self.active_state.as_ref()
    }

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

    pub fn update(&mut self, memory: &mut M) {
        if let Some(id) = &self.active_state {
            self.states.get_mut(&id).unwrap().task.on_update(memory);
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Selector<M, K>
where
    K: Clone + Hash + Eq,
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
    K: Clone + Hash + Eq,
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

pub trait SelectorStatePicker<M = (), K = DefaultKey> {
    fn pick(&mut self, available: &[&K], memory: &M) -> Option<K>;
}

impl<M, K> SelectorStatePicker<M, K> for dyn FnMut(&[&K], &M) -> Option<K> {
    fn pick(&mut self, available: &[&K], memory: &M) -> Option<K> {
        self(available, memory)
    }
}

pub struct ClosureSelectorStatePicker<M = (), K = DefaultKey>(
    Box<dyn FnMut(&[&K], &M) -> Option<K>>,
);

impl<M, K> ClosureSelectorStatePicker<M, K> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnMut(&[&K], &M) -> Option<K> + 'static,
    {
        Self(Box::new(f))
    }
}

impl<M, K> SelectorStatePicker<M, K> for ClosureSelectorStatePicker<M, K> {
    fn pick(&mut self, available: &[&K], memory: &M) -> Option<K> {
        (self.0)(available, memory)
    }
}

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
