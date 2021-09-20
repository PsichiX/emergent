use crate::condition::*;

pub struct CombinatorAll<M> {
    pub conditions: Vec<Box<dyn Condition<M>>>,
}

impl<M> Default for CombinatorAll<M> {
    fn default() -> Self {
        Self { conditions: vec![] }
    }
}

impl<M> CombinatorAll<M> {
    pub fn condition<C>(mut self, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl<M> Condition<M> for CombinatorAll<M> {
    fn validate(&self, memory: &M) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.validate(memory))
    }
}
