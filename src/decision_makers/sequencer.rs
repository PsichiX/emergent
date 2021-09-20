use crate::{condition::*, decision_makers::*, task::*};

pub struct SequencerState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> SequencerState<M> {
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

pub struct Sequencer<M = ()> {
    states: Vec<SequencerState<M>>,
    active_index: Option<usize>,
    looped: bool,
    continuity: bool,
}

impl<M> Sequencer<M> {
    pub fn new(states: Vec<SequencerState<M>>, looped: bool, continuity: bool) -> Self {
        Self {
            states,
            active_index: None,
            looped,
            continuity,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_index.is_some()
    }

    pub fn is_looped(&self) -> bool {
        self.looped
    }

    pub fn does_continue(&self) -> bool {
        self.continuity
    }

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

    pub fn process(&mut self, memory: &mut M) -> bool {
        if self.states.is_empty() {
            return false;
        }
        if let Some(index) = self.active_index {
            if self.states.get(index).unwrap().task.is_locked(memory) {
                return false;
            }
            self.states.get_mut(index).unwrap().task.on_exit(memory);
            let index = if self.looped {
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
            };
            if let Some(index) = index {
                self.states.get_mut(index).unwrap().task.on_enter(memory);
                self.active_index = Some(index);
            } else {
                self.active_index = None;
            }
            return true;
        } else if let Some(index) = self
            .states
            .iter()
            .position(|state| state.condition.validate(memory))
        {
            self.states.get_mut(index).unwrap().task.on_enter(memory);
            self.active_index = Some(index);
            return true;
        }
        if let Some(index) = self.active_index {
            return self.states.get_mut(index).unwrap().task.on_process(memory);
        }
        false
    }

    pub fn update(&mut self, memory: &mut M) {
        if let Some(index) = self.active_index {
            self.states.get_mut(index).unwrap().task.on_update(memory);
        }
    }
}

impl<M> DecisionMaker<M, ()> for Sequencer<M> {
    fn decide(&mut self, memory: &mut M) -> Option<()> {
        self.process(memory);
        Some(())
    }

    fn change_mind(&mut self, _: Option<()>, memory: &mut M) -> bool {
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
