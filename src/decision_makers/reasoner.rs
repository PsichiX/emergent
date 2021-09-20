use crate::{consideration::*, decision_makers::*, task::*, DefaultKey};
use std::{collections::HashMap, hash::Hash};

pub enum ReasonerError<K = DefaultKey> {
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

pub struct ReasonerState<M = ()> {
    consideration: Box<dyn Consideration<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> ReasonerState<M> {
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

    pub fn new_raw(consideration: Box<dyn Consideration<M>>, task: Box<dyn Task<M>>) -> Self {
        Self {
            consideration,
            task,
        }
    }
}

pub struct Reasoner<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    states: HashMap<K, ReasonerState<M>>,
    active_state: Option<K>,
}

impl<M, K> Reasoner<M, K>
where
    K: Clone + Hash + Eq,
{
    pub fn new(states: HashMap<K, ReasonerState<M>>) -> Self {
        Self {
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
    ) -> Result<bool, ReasonerError<K>> {
        if id == self.active_state {
            return Ok(false);
        }
        if let Some(id) = &id {
            if !self.states.contains_key(id) {
                return Err(ReasonerError::StateDoesNotExists(id.clone()));
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
        let new_id = self
            .states
            .iter()
            .map(|(id, state)| (id, state.consideration.score(memory)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap())
            .unwrap()
            .0
            .clone();
        if let Ok(true) = self.change_active_state(Some(new_id), memory, false) {
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

impl<M, K> DecisionMaker<M, K> for Reasoner<M, K>
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

impl<M, K> Task<M> for Reasoner<M, K>
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
