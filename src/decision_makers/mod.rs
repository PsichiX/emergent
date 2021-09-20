pub mod machinery;
pub mod parallelizer;
pub mod planner;
pub mod reasoner;
pub mod selector;
pub mod sequencer;

use crate::DefaultKey;
use std::marker::PhantomData;

pub trait DecisionMaker<M = (), K = DefaultKey> {
    fn decide(&mut self, memory: &mut M) -> Option<K>;
    fn change_mind(&mut self, id: Option<K>, memory: &mut M) -> bool;
}

pub struct NoDecisionMaker<M = (), K = DefaultKey>(PhantomData<(M, K)>);

impl<M, K> Default for NoDecisionMaker<M, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M, K> DecisionMaker<M, K> for NoDecisionMaker<M, K> {
    fn decide(&mut self, _: &mut M) -> Option<K> {
        None
    }

    fn change_mind(&mut self, _: Option<K>, _: &mut M) -> bool {
        false
    }
}
