use crate::condition::*;

pub struct CombinatorAny<M> {
    pub conditions: Vec<Box<dyn Condition<M>>>,
}

impl<M> Default for CombinatorAny<M> {
    fn default() -> Self {
        Self { conditions: vec![] }
    }
}

impl<M> CombinatorAny<M> {
    pub fn condition<C>(mut self, condition: C) -> Self
    where
        C: Condition<M> + 'static,
    {
        self.conditions.push(Box::new(condition));
        self
    }
}

impl<M> Condition<M> for CombinatorAny<M> {
    fn validate(&self, memory: &M) -> bool {
        self.conditions
            .iter()
            .any(|condition| condition.validate(memory))
    }
}
