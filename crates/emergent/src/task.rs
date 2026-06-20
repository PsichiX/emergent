//! Tasks are units of work that perform small and concrete actions in time via memory manipulation.
//!
//! Tasks are used in decision makers as smaller building bocks that can be bundled together into
//! more complex behavior.
//! Each task has a set of life-cycle methods that user apply logic to.
//!
//! The mainly used ones are:
//! - [`Task::on_enter`]
//! - [`Task::on_exit`]
//! - [`Task::on_stop`]
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

/// Describes why task has stopped its work.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskStopReason {
    /// Task reached its desired end.
    Completed,
    /// Task got interrupted before reaching its desired end.
    Cancelled,
    /// Task got replaced by another task or state.
    Replaced,
}

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
    /// Tells if task is locked (it's still running). Used by decision makers to
    /// decide if one can change its state (when current task is not locked).
    #[allow(unused_variables)]
    fn is_locked(&self, memory: &M) -> bool {
        false
    }

    /// Action performed when task starts its work.
    #[allow(unused_variables)]
    fn on_enter(&mut self, memory: &mut M) {}

    /// Action performed when task stops its work.
    #[allow(unused_variables)]
    fn on_exit(&mut self, memory: &mut M) {}

    /// Action performed when task stops its work with reason.
    ///
    /// By default it calls [`Task::on_exit`] for backward compatibility.
    #[allow(unused_variables)]
    fn on_stop(&mut self, memory: &mut M, reason: TaskStopReason) {
        self.on_exit(memory);
    }

    /// Action performed when task is active and gets updated.
    #[allow(unused_variables)]
    fn on_update(&mut self, memory: &mut M) {}

    /// Action performed when task is active but decision maker did not changed
    /// its state. This one is applicable for making hierarchical decision
    /// makers (telling children decision makers to decide on new state, because
    /// some if not all decision makers are tasks).
    ///
    /// Returns `true` if this task decided on new state.
    #[allow(unused_variables)]
    fn on_process(&mut self, memory: &mut M) -> bool {
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
    stop: Option<Box<dyn FnMut(&mut M, TaskStopReason) + Send + Sync>>,
    update: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    process: Option<Box<dyn FnMut(&mut M) -> bool + Send + Sync>>,
}

impl<M> Default for ClosureTask<M> {
    fn default() -> Self {
        Self {
            locked: None,
            enter: None,
            exit: None,
            stop: None,
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

    /// See [`Task::on_stop`]
    pub fn stop<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M, TaskStopReason) + 'static + Send + Sync,
    {
        self.stop = Some(Box::new(f));
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

    fn on_stop(&mut self, memory: &mut M, reason: TaskStopReason) {
        if let Some(f) = &mut self.stop {
            f(memory, reason);
        } else {
            self.on_exit(memory);
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

/// Decides what happens to undo records when a journaled transaction commits.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum TransactionCommitPolicy {
    /// Commit accepts this transaction's changes and discards its undo log.
    ///
    /// Parent transaction rollbacks won't undo changes recorded by this transaction.
    #[default]
    Isolated,
    /// Commit moves this transaction's undo log into its parent transaction.
    ///
    /// Parent transaction rollbacks can still undo changes recorded by this transaction.
    MergeIntoParent,
}

/// Stack of undo records used by journaled transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionJournal<U> {
    frames: Vec<Vec<U>>,
}

impl<U> Default for TransactionJournal<U> {
    fn default() -> Self {
        Self { frames: Vec::new() }
    }
}

impl<U> TransactionJournal<U> {
    /// Starts new transaction frame.
    pub fn begin(&mut self) {
        self.frames.push(Vec::new());
    }

    /// Records undo entry in currently active transaction frame.
    ///
    /// Returns `false` when there is no active transaction frame.
    pub fn record(&mut self, undo: U) -> bool {
        let Some(frame) = self.frames.last_mut() else {
            return false;
        };
        frame.push(undo);
        true
    }

    /// Commits currently active transaction frame.
    pub fn commit(&mut self, policy: TransactionCommitPolicy) {
        let Some(frame) = self.frames.pop() else {
            return;
        };

        if policy == TransactionCommitPolicy::MergeIntoParent
            && let Some(parent) = self.frames.last_mut()
        {
            parent.extend(frame);
        }
    }

    /// Rolls back currently active transaction frame.
    pub fn rollback(&mut self) -> impl Iterator<Item = U> {
        self.frames.pop().unwrap_or_default().into_iter().rev()
    }

    /// Tells if there is an active transaction frame.
    pub fn is_active(&self) -> bool {
        !self.frames.is_empty()
    }

    /// Returns number of active transaction frames.
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

/// Memory that stores and applies journaled transaction undo records.
pub trait TransactionalMemory {
    /// Domain-specific undo record.
    type Undo;

    /// Starts new transaction frame.
    fn begin_transaction(&mut self);

    /// Commits currently active transaction frame.
    fn commit_transaction(&mut self, policy: TransactionCommitPolicy);

    /// Rolls back currently active transaction frame.
    fn rollback_transaction(&mut self);

    /// Records undo entry in currently active transaction frame.
    fn record_undo(&mut self, undo: Self::Undo);
}

/// Wrapper around task that creates transaction scope with undo records stored in memory.
pub struct JournaledTransactionTask<M = ()> {
    task: Box<dyn Task<M>>,
    active: bool,
    commit_policy: TransactionCommitPolicy,
}

impl<M> JournaledTransactionTask<M> {
    /// Constructs journaled transaction wrapper around task.
    pub fn new<T>(task: T) -> Self
    where
        T: Task<M> + 'static,
    {
        Self::new_raw(Box::new(task))
    }

    /// Constructs journaled transaction wrapper around raw task.
    pub fn new_raw(task: Box<dyn Task<M>>) -> Self {
        Self {
            task,
            active: false,
            commit_policy: TransactionCommitPolicy::default(),
        }
    }

    /// Sets commit policy.
    pub fn commit_policy(mut self, policy: TransactionCommitPolicy) -> Self {
        self.commit_policy = policy;
        self
    }

    /// Returns commit policy.
    pub fn get_commit_policy(&self) -> TransactionCommitPolicy {
        self.commit_policy
    }

    /// Returns immutable access to wrapped task.
    pub fn task(&self) -> &dyn Task<M> {
        self.task.as_ref()
    }

    /// Returns mutable access to wrapped task.
    pub fn task_mut(&mut self) -> &mut dyn Task<M> {
        self.task.as_mut()
    }
}

impl<M> Task<M> for JournaledTransactionTask<M>
where
    M: TransactionalMemory,
{
    fn is_locked(&self, memory: &M) -> bool {
        self.task.is_locked(memory)
    }

    fn on_enter(&mut self, memory: &mut M) {
        memory.begin_transaction();
        self.task.on_enter(memory);
        self.active = true;
    }

    fn on_exit(&mut self, memory: &mut M) {
        self.on_stop(memory, TaskStopReason::Cancelled);
    }

    fn on_stop(&mut self, memory: &mut M, reason: TaskStopReason) {
        self.task.on_stop(memory, reason);

        if !self.active {
            return;
        }

        match reason {
            TaskStopReason::Completed => {
                memory.commit_transaction(self.commit_policy);
            }
            TaskStopReason::Cancelled | TaskStopReason::Replaced => {
                memory.rollback_transaction();
            }
        }

        self.active = false;
    }

    fn on_update(&mut self, memory: &mut M) {
        self.task.on_update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.task.on_process(memory)
    }
}

impl<M> std::fmt::Debug for JournaledTransactionTask<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JournaledTransactionTask")
            .field("active", &self.active)
            .field("commit_policy", &self.commit_policy)
            .finish()
    }
}

/// Wrapper around task that creates transaction scope with custom begin/commit/rollback hooks.
#[allow(clippy::type_complexity)]
pub struct TransactionScopeTask<M = ()> {
    task: Box<dyn Task<M>>,
    active: bool,
    begin: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    commit: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
    rollback: Option<Box<dyn FnMut(&mut M) + Send + Sync>>,
}

impl<M> TransactionScopeTask<M> {
    /// Constructs transaction wrapper around task.
    pub fn new<T>(task: T) -> Self
    where
        T: Task<M> + 'static,
    {
        Self::new_raw(Box::new(task))
    }

    /// Constructs transaction wrapper around raw task.
    pub fn new_raw(task: Box<dyn Task<M>>) -> Self {
        Self {
            task,
            active: false,
            begin: None,
            commit: None,
            rollback: None,
        }
    }

    /// Sets begin transaction hook.
    pub fn begin<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.begin = Some(Box::new(f));
        self
    }

    /// Sets commit transaction hook.
    pub fn commit<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.commit = Some(Box::new(f));
        self
    }

    /// Sets rollback transaction hook.
    pub fn rollback<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static + Send + Sync,
    {
        self.rollback = Some(Box::new(f));
        self
    }

    /// Returns immutable access to wrapped task.
    pub fn task(&self) -> &dyn Task<M> {
        self.task.as_ref()
    }

    /// Returns mutable access to wrapped task.
    pub fn task_mut(&mut self) -> &mut dyn Task<M> {
        self.task.as_mut()
    }
}

impl<M> Task<M> for TransactionScopeTask<M> {
    fn is_locked(&self, memory: &M) -> bool {
        self.task.is_locked(memory)
    }

    fn on_enter(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.begin {
            f(memory);
        }
        self.task.on_enter(memory);
        self.active = true;
    }

    fn on_exit(&mut self, memory: &mut M) {
        self.on_stop(memory, TaskStopReason::Cancelled);
    }

    fn on_stop(&mut self, memory: &mut M, reason: TaskStopReason) {
        self.task.on_stop(memory, reason);

        if !self.active {
            return;
        }

        match reason {
            TaskStopReason::Completed => {
                if let Some(f) = &mut self.commit {
                    f(memory);
                }
            }
            TaskStopReason::Cancelled | TaskStopReason::Replaced => {
                if let Some(f) = &mut self.rollback {
                    f(memory);
                }
            }
        }

        self.active = false;
    }

    fn on_update(&mut self, memory: &mut M) {
        self.task.on_update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.task.on_process(memory)
    }
}

impl<M> std::fmt::Debug for TransactionScopeTask<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionScopeTask")
            .field("active", &self.active)
            .finish()
    }
}
