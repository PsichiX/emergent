use std::marker::PhantomData;

pub trait Task<M = ()> {
    fn is_locked(&self, _memory: &M) -> bool {
        false
    }
    fn on_enter(&mut self, _memory: &mut M) {}
    fn on_exit(&mut self, _memory: &mut M) {}
    fn on_update(&mut self, _memory: &mut M) {}
    fn on_process(&mut self, _memory: &mut M) -> bool {
        false
    }
}

pub struct NoTask<M = ()>(PhantomData<M>);

impl<M> Default for NoTask<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M> Task<M> for NoTask<M> {}

impl<M> Task<M> for dyn FnMut(&M) {
    fn on_enter(&mut self, memory: &mut M) {
        self(memory);
    }
}

pub struct ClosureTask<M = ()> {
    is_locked: Option<Box<dyn Fn(&M) -> bool>>,
    on_enter: Option<Box<dyn FnMut(&mut M)>>,
    on_exit: Option<Box<dyn FnMut(&mut M)>>,
    on_update: Option<Box<dyn FnMut(&mut M)>>,
    on_process: Option<Box<dyn FnMut(&mut M) -> bool>>,
}

impl<M> Default for ClosureTask<M> {
    fn default() -> Self {
        Self {
            is_locked: None,
            on_enter: None,
            on_exit: None,
            on_update: None,
            on_process: None,
        }
    }
}

impl<M> ClosureTask<M> {
    #[allow(clippy::wrong_self_convention)]
    pub fn is_locked<F>(mut self, f: F) -> Self
    where
        F: Fn(&M) -> bool + 'static,
    {
        self.is_locked = Some(Box::new(f));
        self
    }

    pub fn on_enter<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static,
    {
        self.on_enter = Some(Box::new(f));
        self
    }

    pub fn on_exit<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static,
    {
        self.on_exit = Some(Box::new(f));
        self
    }

    pub fn on_update<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) + 'static,
    {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn on_process<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut M) -> bool + 'static,
    {
        self.on_process = Some(Box::new(f));
        self
    }
}

impl<M> Task<M> for ClosureTask<M> {
    fn is_locked(&self, memory: &M) -> bool {
        self.is_locked
            .as_ref()
            .map(|f| f(memory))
            .unwrap_or_default()
    }

    fn on_enter(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.on_enter {
            f(memory)
        }
    }

    fn on_exit(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.on_exit {
            f(memory)
        }
    }

    fn on_update(&mut self, memory: &mut M) {
        if let Some(f) = &mut self.on_update {
            f(memory)
        }
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.on_process
            .as_mut()
            .map(|f| f(memory))
            .unwrap_or_default()
    }
}
