//! LOD builder.

use crate::{condition::*, decision_makers::selector::*, task::*};

/// Wrapper around memory that holds additional information about current level of details.
pub struct LodMemory<M = ()> {
    pub lod_level: usize,
    pub memory: M,
}

impl<M> std::fmt::Debug for LodMemory<M>
where
    M: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LodMemory")
            .field("lod_level", &self.lod_level)
            .field("memory", &self.memory)
            .finish()
    }
}

/// Allows to run different decision making depending on the level of details set in memory.
///
/// Useful to optimize AI processing to for example run narow phase logic when agent is near the
/// player and run broad phase logic when agent runs in the background.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// const DELTA_TIME: Scalar = 1.0;
///
/// struct Memory {
///     time_since_last_meal: Scalar,
///     hunger: Scalar,
/// }
///
/// let mut lod = Lod::default()
///     // level 0 means agent is not in area and we optimize AI processing by not doing any work,
///     // so on task exit we just estimate how much more hungry agent can get during task time.
///     .level(ClosureTask::default().exit(|m: &mut LodMemory<Memory>| {
///         m.memory.hunger -= m.memory.time_since_last_meal;
///         println!("* Background hunger estimation: {}", m.memory.hunger);
///     }))
///     // level 1 means agent is in area and we want to accurately change its hunger level.
///     .level(ClosureTask::default().update(|m: &mut LodMemory<Memory>| {
///         m.memory.hunger -= DELTA_TIME;
///         println!("* Foreground hunger calculation: {}", m.memory.hunger);
///     }))
///     .build();
///
/// let mut memory = LodMemory {
///     lod_level: 0,
///     memory: Memory {
///         time_since_last_meal: 0.0,
///         hunger: 10.0,
///     },
/// };
///
/// // we start with agent running in the background.
/// assert_eq!(lod.active_state(), None);
/// assert_eq!(lod.process(&mut memory), true);
/// assert_eq!(lod.active_state(), Some(&0));
/// // agent will now run in foreground and we assume 5 seconds have passed since last meal.
/// memory.lod_level = 1;
/// memory.memory.time_since_last_meal = 5.0;
/// assert_eq!(lod.process(&mut memory), true);
/// assert_eq!(lod.active_state(), Some(&1));
/// assert_eq!(memory.memory.hunger, 5.0);
/// lod.update(&mut memory);
/// assert_eq!(memory.memory.hunger, 4.0);
/// ```
pub struct Lod<M = ()>(Vec<Box<dyn Task<LodMemory<M>>>>);

impl<M> Default for Lod<M> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<M> Lod<M> {
    /// Constructs new LOD with list of tasks as LOD levels.
    pub fn new(levels: Vec<Box<dyn Task<LodMemory<M>>>>) -> Self {
        Self(levels)
    }

    /// Adds new level with task.
    pub fn level<T>(mut self, task: T) -> Self
    where
        T: Task<LodMemory<M>> + 'static,
    {
        self.0.push(Box::new(task));
        self
    }

    /// Adds new level with task.
    pub fn level_raw(mut self, task: Box<dyn Task<LodMemory<M>>>) -> Self {
        self.0.push(task);
        self
    }

    /// Consumes this builder and builds selector decision maker that will switch between levels.
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

impl<M> std::fmt::Debug for Lod<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lod").finish()
    }
}
