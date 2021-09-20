use crate::{
    condition::*,
    decision_makers::{selector::*, sequencer::*},
    task::*,
};

pub enum BehaviorTree<M = ()> {
    Sequence {
        condition: Box<dyn Condition<M>>,
        nodes: Vec<BehaviorTree<M>>,
    },
    Selector {
        condition: Box<dyn Condition<M>>,
        nodes: Vec<BehaviorTree<M>>,
    },
    State {
        condition: Box<dyn Condition<M>>,
        task: Box<dyn Task<M>>,
    },
}

impl<M> BehaviorTree<M> {
    pub fn sequence<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Sequence {
            condition: Box::new(condition),
            nodes: vec![],
        }
    }

    pub fn sequence_nodes<C>(condition: C, nodes: Vec<BehaviorTree<M>>) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Sequence {
            condition: Box::new(condition),
            nodes,
        }
    }

    pub fn selector<C>(condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Selector {
            condition: Box::new(condition),
            nodes: vec![],
        }
    }

    pub fn selector_nodes<C>(condition: C, nodes: Vec<BehaviorTree<M>>) -> Self
    where
        C: Condition<M> + 'static,
    {
        Self::Selector {
            condition: Box::new(condition),
            nodes,
        }
    }

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

    pub fn node(mut self, node: BehaviorTree<M>) -> Self {
        match &mut self {
            Self::Sequence { nodes, .. } => nodes.push(node),
            Self::Selector { nodes, .. } => nodes.push(node),
            Self::State { .. } => {}
        }
        self
    }

    pub fn build(self) -> Box<dyn Task<M>>
    where
        M: 'static,
    {
        self.consume().1
    }

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
            Self::State { condition, task } => (condition, task),
        }
    }
}
