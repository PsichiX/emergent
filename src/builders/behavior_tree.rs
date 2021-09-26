//! Behavior tree builder.

use crate::{
    condition::*,
    decision_makers::{parallelizer::*, selector::*, sequencer::*},
    task::*,
};

/// Wrapper around task produced by [`BehaviorTree`] builder.
pub struct BehaviorTreeTask<M = ()>(pub Box<dyn Task<M>>);

impl<M> Task<M> for BehaviorTreeTask<M> {
    fn is_locked(&self, memory: &M) -> bool {
        self.0.is_locked(memory)
    }

    fn on_enter(&mut self, memory: &mut M) {
        self.0.on_enter(memory);
    }

    fn on_exit(&mut self, memory: &mut M) {
        self.0.on_exit(memory);
    }

    fn on_update(&mut self, memory: &mut M) {
        self.0.on_update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.0.on_process(memory)
    }
}

impl<M> std::fmt::Debug for BehaviorTreeTask<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehaviorTreeTask").finish()
    }
}

/// Builds hierarchy of decision makers that work together just as behavior tree.
///
/// Behavior trees are commonly used AI technique that alows to organize AI choices and sequences
/// of tasks that agent has to perform in a tree manner. Tree is always executed from the top-left
/// to the bottom-right, which means it starts at the root and when finds branching (sequence or
/// selector) it will try to execute each children in a set from first to last.
///
/// - __Sequence__ (equivalent of boolean AND operation):
///
///   Runs its children one by one until one fails and sequence stops there.
///
/// - __Selector__ (equivalent of boolean OR operation):
///
///   Will always run only one child node at a time, first one that's condition returns true.
///
/// It's worth noting that sequences and selectors won't switch their nodes as long as currently
/// running node reports [`Task::is_locked`] true (this means node is running and nothing should
/// interrupt its work).
///
/// How it works
/// ---
/// Imagine you have tree like this:
/// - selector:
///   - sequence (is hungry?):
///     - Find food (is there food nearby?)
///     - Eat (is found food edible?)
///   - sequence (is low energy?):
///     - Find safe place (is there a cave nearby?)
///     - Sleep (is there no bear nearby?)
///   - Do nothing (always succeeds)
///
/// We are gonna run this tree against initial memory state:
/// - hungry: false
/// - food nearby: true
/// - edible food: true
/// - low energy: false
/// - cave nearby: false
/// - bear nearby: true
///
/// Starting from the root selector we get two options to test their conditions, first one won't run
/// because agent is not hungry, then second one also won't run because agent doesn't have low energy.
/// What's left is __do nothing__ which always secceeds.
///
/// Now we change our state:
/// - hungry: false
/// - food nearby: true
/// - edible food: true
/// - __low energy: true__
/// - cave nearby: false
/// - bear nearby: true
///
/// We again start at selector and test its children conditions: first node won't run because agent
/// still isn't hungry, but we will run second node because agent __has low energy__.
/// Now since we have activated sleeping sequence we try to run each child node as long as they
/// succeed: we can't find safe place since there is no cave nearby, and because that node has
/// failed it ended the sequence, so we won't try to sleep.
///
/// We change our state again:
/// - __hungry: true__
/// - food nearby: true
/// - edible food: true
/// - low energy: true
/// - cave nearby: false
/// - bear nearby: true
///
/// We are gonna run forst sequence since agent __is hungry__, in that sequence we __find food nearby__
/// and we __eat it__ since it is edible, and that has completed the sequence.
///
/// See: [https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control)](https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control))
///
/// # Example
/// ```
/// use emergent::prelude::*;
///
/// struct Memory {
///     mode: bool,
///     counter: usize,
/// }
///
/// struct Countdown(pub usize);
///
/// impl Task<Memory> for Countdown {
///     fn is_locked(&self, memory: &Memory) -> bool {
///         memory.counter > 0
///     }
///
///     fn on_enter(&mut self, memory: &mut Memory) {
///         memory.counter = self.0;
///     }
///
///     fn on_update(&mut self, memory: &mut Memory) {
///         memory.counter = memory.counter.max(1) - 1;
///     }
/// }
///
/// struct FlipMode;
///
/// impl Task<Memory> for FlipMode {
///     fn on_enter(&mut self, memory: &mut Memory) {
///         memory.mode = !memory.mode;
///     }
/// }
///
/// struct IsMode(pub bool);
///
/// impl Condition<Memory> for IsMode {
///     fn validate(&self, memory: &Memory) -> bool {
///         memory.mode == self.0
///     }
/// }
///
/// // we define a tree that will perform ping-pong with delay:
/// // first we wait 2 turns, flip memory state, then wait 1 turn and flip back memory state.
/// let mut tree = BehaviorTree::selector(true)
///     .node(
///         BehaviorTree::sequence(IsMode(true))
///             .node(BehaviorTree::state(true, Countdown(2)))
///             .node(BehaviorTree::state(true, FlipMode)),
///     )
///     .node(
///         BehaviorTree::sequence(IsMode(false))
///             .node(BehaviorTree::state(true, Countdown(1)))
///             .node(BehaviorTree::state(true, FlipMode)),
///     )
///     .build();
///
/// let mut memory = Memory {
///     mode: true,
///     counter: 0,
/// };
///
/// assert_eq!(tree.on_process(&mut memory), true);
/// assert_eq!(memory.mode, true);
/// assert_eq!(memory.counter, 2);
///
/// assert_eq!(tree.on_process(&mut memory), false);
/// tree.on_update(&mut memory);
/// assert_eq!(memory.mode, true);
/// assert_eq!(memory.counter, 1);
///
/// assert_eq!(tree.on_process(&mut memory), false);
/// tree.on_update(&mut memory);
/// assert_eq!(memory.mode, true);
/// assert_eq!(memory.counter, 0);
///
/// assert_eq!(tree.on_process(&mut memory), true);
///
/// assert_eq!(tree.on_process(&mut memory), true);
/// assert_eq!(memory.mode, false);
/// assert_eq!(memory.counter, 1);
///
/// assert_eq!(tree.on_process(&mut memory), false);
/// tree.on_update(&mut memory);
/// assert_eq!(memory.mode, false);
/// assert_eq!(memory.counter, 0);
///
/// assert_eq!(tree.on_process(&mut memory), true);
/// assert_eq!(memory.mode, true);
/// ```
pub enum BehaviorTree<M = ()> {
    /// Sequence node runs its children as long as they succeed (boolean AND operation).
    ///
    /// It locks changes of higher nodes as long as its running child node is locked.
    Sequence {
        condition: Box<dyn Condition<M>>,
        nodes: Vec<BehaviorTree<M>>,
    },
    /// Selector node runs its first children that succeeds (boolean OR operation).
    ///
    /// It locks changes of higher nodes as long as its running child node is locked.
    Selector {
        condition: Box<dyn Condition<M>>,
        nodes: Vec<BehaviorTree<M>>,
    },
    /// Parallel node runs all its children at the same time.
    ///
    /// It locks changes of higher nodes as long as any of its running children nodes is locked.
    Parallel {
        condition: Box<dyn Condition<M>>,
        nodes: Vec<BehaviorTree<M>>,
    },
    /// State runs certain task.
    ///
    /// it locks changes of higher nodes as long as its task is locked.
    State {
        condition: Box<dyn Condition<M>>,
        task: Box<dyn Task<M>>,
    },
}

impl<M> BehaviorTree<M> {
    /// Constructs sequence node with condition.
    pub fn sequence<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Sequence {
            condition: Box::new(condition),
            nodes: vec![],
        }
    }

    /// Constructs sequence node with condition and child nodes.
    pub fn sequence_nodes<C>(condition: C, nodes: Vec<BehaviorTree<M>>) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Sequence {
            condition: Box::new(condition),
            nodes,
        }
    }

    /// Constructs selector node with condition.
    pub fn selector<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Selector {
            condition: Box::new(condition),
            nodes: vec![],
        }
    }

    /// Constructs selector node with condition and child nodes.
    pub fn selector_nodes<C>(condition: C, nodes: Vec<BehaviorTree<M>>) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Selector {
            condition: Box::new(condition),
            nodes,
        }
    }

    /// Constructs parallel node with condition.
    pub fn parallel<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Parallel {
            condition: Box::new(condition),
            nodes: vec![],
        }
    }

    /// Constructs parallel node with condition and child nodes.
    pub fn parallel_nodes<C>(condition: C, nodes: Vec<BehaviorTree<M>>) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Selector {
            condition: Box::new(condition),
            nodes,
        }
    }

    /// Constructs state node with condition and task.
    pub fn state<C, T>(condition: C, task: T) -> Self
    where
        C: Condition<M> + 'static,
        T: Task<M> + 'static,
    {
        Self::State {
            condition: Box::new(condition),
            task: Box::new(task),
        }
    }

    /// Adds child node to this branch (when called on state node it does nothing).
    pub fn node(mut self, node: BehaviorTree<M>) -> Self {
        match &mut self {
            Self::Sequence { nodes, .. } => nodes.push(node),
            Self::Selector { nodes, .. } => nodes.push(node),
            Self::Parallel { nodes, .. } => nodes.push(node),
            Self::State { .. } => {}
        }
        self
    }

    /// Consumes this builder and builds final tree as a task.
    pub fn build(self) -> BehaviorTreeTask<M>
    where
        M: 'static,
    {
        BehaviorTreeTask(self.consume().1)
    }

    /// Consumes this builder and returns its rot condition and root task.
    pub fn consume(self) -> (Box<dyn Condition<M>>, Box<dyn Task<M>>)
    where
        M: 'static,
    {
        match self {
            Self::Sequence { condition, nodes } => {
                let states = nodes
                    .into_iter()
                    .map(|node| {
                        let (condition, task) = node.consume();
                        SequencerState::new_raw(condition, task)
                    })
                    .collect();
                let sequencer = Sequencer::new(states, false, false);
                (condition, Box::new(sequencer))
            }
            Self::Selector { condition, nodes } => {
                let states = nodes
                    .into_iter()
                    .enumerate()
                    .map(|(index, node)| {
                        let (condition, task) = node.consume();
                        (index, SelectorState::new_raw(condition, task))
                    })
                    .collect();
                let selector = Selector::new(OrderedSelectorStatePicker::First, states);
                (condition, Box::new(selector))
            }
            Self::Parallel { condition, nodes } => {
                let states = nodes
                    .into_iter()
                    .map(|node| {
                        let (condition, task) = node.consume();
                        ParallelizerState::new_raw(condition, task)
                    })
                    .collect();
                let selector = Parallelizer::new(states);
                (condition, Box::new(selector))
            }
            Self::State { condition, task } => (condition, task),
        }
    }
}

impl<M> std::fmt::Debug for BehaviorTree<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sequence { nodes, .. } => {
                f.debug_struct("Sequence").field("nodes", &nodes).finish()
            }
            Self::Selector { nodes, .. } => {
                f.debug_struct("Selector").field("nodes", &nodes).finish()
            }
            Self::Parallel { nodes, .. } => {
                f.debug_struct("Parallel").field("nodes", &nodes).finish()
            }
            Self::State { .. } => f.debug_struct("State").finish(),
        }
    }
}
