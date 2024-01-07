//! Tasks are units of work that perform small and concrete actions in time via memory manipulation.
//!
//! Tasks are used in decision makers as smaller building bocks that can be bundled together into
//! more complex behavior.
//! Each task has a set of life-cycle methods that user apply logic to.
//!
//! The mainly used ones are:
//! - [`Task::on_enter`]
//! - [`Task::on_exit`]
//! - [`Task::on_update`]
//!
//! By default all life-cycle methods are auto-implemented so user when defining a new task, focuses
//! only on implementing these methods that are needed.
//!
//! Read more about creating custom tasks at [`Task`].
//!
//! There are two generic tasks created for the user:
//! - [`NoTask`]: used as empty task, a task without any work, you can use it in places where AI
//!   should simply do nothing while that task is active.
//! - [`ClosureTask`]: a wrapper around closure-based tasks where each life-cycle method is provided
//!   by the user as separate closures, best for prototyping or making small non-repetitive logic.

/// Task represent unit of work, an action performed in a time.
///
/// Tasks can only manipulate what's in the memory passed to their life-cycle methods so common way
/// of manipulating the world with tasks is to make memory type that can either command the world
/// or even better, put triggers in the memory before running decision maker, then after that read
/// what changed in the memory and apply changes to the world.
///
/// Tasks are managed by calling their life-cycle methods by the decision makers so implement these
/// if you need to do something during that life-cycle phase.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// struct Increment;
///
/// impl Task<Memory> for Increment {
///     fn on_enter(&mut self, memory: &mut Memory) {
///         memory.counter += 1;
///     }
/// }
///
/// let mut memory = Memory { counter: 1 };
/// Increment.on_enter(&mut memory);
/// assert_eq!(memory.counter, 2);
/// ```
pub trait Task<M = ()>: Send + Sync {
    /// Tells if task is locked (it's still running). Used by decision makers to tell if one can
    /// change its state (when current task is not locked).
    fn is_locked(&self, _memory: &M) -> bool {
        false
    }

    /// Action performed when task starts its work.
    fn on_enter(&mut self, _memory: &mut M) {}

    /// Action performed when task stops its work.
    fn on_exit(&mut self, _memory: &mut M) {}

    /// Action performed when task is active and gets updated.
    fn on_update(&mut self, _memory: &mut M) {}

    /// Action performed when task is active but decision maker did not changed its state.
    /// This one is applicable for making hierarchical decision makers (telling children decision
    /// makers to decide on new state, because some if not all decision makers are tasks).
    ///
    /// Returns `true` if this task decided on new state.
    fn on_process(&mut self, _memory: &mut M) -> bool {
        false
    }
}

/// Task that represent no work. Use it when AI has to do nothing.
#[derive(Debug, Default, Copy, Clone)]
pub struct NoTask;

impl<M> Task<M> for NoTask {}

/// Task thet wraps closures for each life-cycle method.
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory { counter: usize }
///
/// let mut memory = Memory { counter: 1 };
/// let mut task = ClosureTask::default().enter(|memory: &mut Memory| memory.counter += 1);
/// task.on_enter(&mut memory);
/// assert_eq!(memory.counter, 2);
/// ```
#[allow(clippy::type_complexity)]
pub struct ClosureTask<M = ()> {
    locked: Option<Box<dyn Fn(&M) -> bool + Send + Sync>>,
    enter: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    exit: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    update: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    process: Option<Box<dyn FnMut(&mut M) -> bool + Send + Sync>>,
}

impl<M> Default for ClosureTask<M> {
    fn default() -> Self {
        Self {
            locked: None,
            enter: None,
            exit: None,
            update: None,
            process: None,
        }
    }
}

impl<M> ClosureTask<M> {
    /// See [`Task::is_locked`]
    pub fn locked<F>(mut self, f: F) -> Self
    where
        F: Fn(&M) -> bool + 'static + Send + Sync,
    {
        self.locked = Some(Box::new(f));
        self
    }

    /// See [`Task::on_enter`]
    pub fn enter<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.enter = Some(Box::new(f));
        self
    }

    /// See [`Task::on_exit`]
    pub fn exit<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.exit = Some(Box::new(f));
        self
    }

    /// See [`Task::on_update`]
    pub fn update<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.update = Some(Box::new(f));
        self
    }

    /// See [`Task::on_process`]
    pub fn process<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) -> bool + 'static + Send + Sync,
    {
        self.process = Some(Box::new(f));
        self
    }
}

impl<M> Task<M> for ClosureTask<M> {
    fn is_locked(&self, memory: &M) -> bool {
        self.locked.as_ref().map(|f| f(memory)).unwrap_or_default()
    }

    fn on_enter(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.enter {
            f(memory)
        }
    }

    fn on_exit(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.exit {
            f(memory)
        }
    }

    fn on_update(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.update {
            f(memory)
        }
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.process.as_mut().map(|f| f(memory)).unwrap_or_default()
    }
}

impl<M> std::fmt::Debug for ClosureTask<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureTask").finish()
    }
}
