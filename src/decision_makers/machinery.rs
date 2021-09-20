use crate::{condition::*, decision_makers::*, task::*, DefaultKey};
use std::{collections::HashMap, hash::Hash};

pub enum MachineryError<K = DefaultKey> {
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

pub struct MachineryChange<M = (), K = DefaultKey> {
    pub to: K,
    pub condition: Box<dyn Condition<M>>,
}

impl<M, K> MachineryChange<M, K> {
    pub fn new<C>(to: K, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self {
            to,
            condition: Box::new(condition),
        }
    }

    pub fn new_raw<C>(to: K, condition: Box<dyn Condition<M>>) -> Self {
        Self { to, condition }
    }

    pub fn validate(&self, memory: &M) -> bool {
        self.condition.validate(memory)
    }
}

pub struct MachineryState<M = (), K = DefaultKey> {
    task: Box<dyn Task<M>>,
    changes: Vec<MachineryChange<M, K>>,
}

impl<M, K> MachineryState<M, K> {
    pub fn new<T>(task: T, changes: Vec<MachineryChange<M, K>>) -> Self
    where
        T: Task<M> + 'static,
    {
        Self {
            task: Box::new(task),
            changes,
        }
    }

    pub fn new_raw<T>(task: Box<dyn Task<M>>, changes: Vec<MachineryChange<M, K>>) -> Self {
        Self { task, changes }
    }
}

pub struct Machinery<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    states: HashMap<K, MachineryState<M, K>>,
    active_state: Option<K>,
}

impl<M, K> Machinery<M, K>
where
    K: Clone + Hash + Eq,
{
    pub fn new(states: HashMap<K, MachineryState<M, K>>) -> Self {
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

    pub fn update(&mut self, memory: &mut M) {
        if let Some(id) = &self.active_state {
            self.states.get_mut(&id).unwrap().task.on_update(memory);
        }
    }
}

impl<M, K> DecisionMaker<M, K> for Machinery<M, K>
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

impl<M, K> Task<M> for Machinery<M, K>
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
