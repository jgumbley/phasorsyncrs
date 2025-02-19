use std::thread;

use crate::{ui::run_state_inspector, SharedState};

pub trait Scheduler {
    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    fn spawn_state_inspector(&self, shared_state: &SharedState)
    where
        Self: Sized,
    {
        let inspector_state = shared_state.clone();
        self.spawn(move || {
            run_state_inspector(inspector_state);
        });
    }
}

pub struct ThreadScheduler;

impl ThreadScheduler {
    pub fn new() -> Self {
        ThreadScheduler
    }
}

impl Default for ThreadScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for ThreadScheduler {
    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = thread::spawn(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_thread_scheduler_spawn() {
        let scheduler = ThreadScheduler::new();
        let flag = Arc::new(Mutex::new(false));
        let flag_clone = flag.clone();

        scheduler.spawn(move || {
            let mut flag = flag_clone.lock().unwrap();
            *flag = true;
        });

        // Give the thread a moment to execute
        thread::sleep(Duration::from_millis(10));
        assert!(*flag.lock().unwrap());
    }
}
