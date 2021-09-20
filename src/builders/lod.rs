use crate::{condition::*, decision_makers::selector::*, task::*};

pub struct LodMemory<M = ()> {
    pub lod_level: usize,
    pub memory: M,
}

pub struct Lod<M = ()>(Vec<Box<dyn Task<LodMemory<M>>>>);

impl<M> Default for Lod<M> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<M> Lod<M> {
    pub fn new(levels: Vec<Box<dyn Task<LodMemory<M>>>>) -> Self {
        Self(levels)
    }

    pub fn level<T>(mut self, task: T) -> Self
    where
        T: Task<LodMemory<M>> + 'static,
    {
        self.0.push(Box::new(task));
        self
    }

    pub fn level_raw(mut self, task: Box<dyn Task<LodMemory<M>>>) -> Self {
        self.0.push(task);
        self
    }

    pub fn build(self) -> Selector<LodMemory<M>, usize>
    where
        M: 'static,
    {
        let states = self
            .0
            .into_iter()
            .enumerate()
            .map(|(index, task)| {
                let state = SelectorState::new_raw(
                    Box::new(ClosureCondition::new(move |memory: &LodMemory<M>| {
                        memory.lod_level == index
                    })),
                    task,
                );
                (index, state)
            })
            .collect();
        Selector::new(OrderedSelectorStatePicker::First, states)
    }
}
