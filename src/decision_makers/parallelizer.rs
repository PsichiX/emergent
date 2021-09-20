use crate::{condition::*, decision_makers::*, task::*};

pub struct ParallelizerState<M = ()> {
    condition: Box<dyn Condition<M>>,
    task: Box<dyn Task<M>>,
}

impl<M> ParallelizerState<M> {
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

pub struct Parallelizer<M = ()> {
    states: Vec<(ParallelizerState<M>, bool)>,
}

impl<M> Parallelizer<M> {
    pub fn new(states: Vec<ParallelizerState<M>>) -> Self {
        Self {
            states: states.into_iter().map(|state| (state, false)).collect(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.states.iter().any(|(_, active)| *active)
    }

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

    pub fn update(&mut self, memory: &mut M) {
        for (state, active) in &mut self.states {
            if *active {
                state.task.on_update(memory);
            }
        }
    }
}

impl<M> DecisionMaker<M, ()> for Parallelizer<M> {
    fn decide(&mut self, memory: &mut M) -> Option<()> {
        self.process(memory);
        Some(())
    }

    fn change_mind(&mut self, _: Option<()>, memory: &mut M) -> bool {
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
