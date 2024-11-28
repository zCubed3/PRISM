use super::*;
use std::sync::{Arc, RwLock};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SemaphoreState {
    NotBegan,
    Working,
    Finished,
}

#[derive(Clone)]
pub struct Semaphore {
    _owner: *const Dispatcher,
    _flag: Arc<RwLock<SemaphoreState>>,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(owner: &Dispatcher) -> Semaphore {
        return Semaphore {
            _owner: owner,
            _flag: Arc::new(RwLock::new(SemaphoreState::NotBegan)),
        };
    }

    pub fn set_flag(&self, flag: SemaphoreState) {
        let mut flag_guard = self._flag.write().expect("Failed to write flag!");
        *flag_guard = flag;
    }

    pub fn get_flag(&self) -> SemaphoreState {
        let flag_guard = self._flag.read().expect("Failed to read flag!");
        return *flag_guard;
    }

    pub fn wait(&self) {
        while self.get_flag() != SemaphoreState::Finished {}
    }
}
