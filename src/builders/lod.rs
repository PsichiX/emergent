use crate::{condition::*, decision_makers::selector::*, task::*};

/// Wrapper around memory that holds additional information about current level of details.
pub struct LodMemory<M = ()> {
    pub lod_level: usize,
    pub memory: M,
}

/// Allows to run different decision making depending on the level of details.
///
/// Useful to optimize AI processing to for example run narow phase logic when agent is near the
/// player and run broad phase logic when agent runs in the background.
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
