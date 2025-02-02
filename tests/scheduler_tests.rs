#[cfg(test)]
mod tests {
    use phasorsyncrs::{create_scheduler, create_shared_state, Scheduler};
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        sync::Arc,
        thread,
        time::Duration,
    };

    #[test]
    fn test_scheduler_with_transport() {
        let scheduler = create_scheduler();
        let shared_state = create_shared_state();
        let task_completed = Arc::new(AtomicBool::new(false));
        let task_completed_clone = task_completed.clone();

        // Spawn a task that modifies transport state
        scheduler.spawn(move || {
            let mut transport = shared_state.lock().unwrap();
            transport.set_tempo(120.0);
            task_completed_clone.store(true, Ordering::SeqCst);
        });

        // Give the task time to complete
        thread::sleep(Duration::from_millis(50));

        assert!(
            task_completed.load(Ordering::SeqCst),
            "Task should have completed"
        );
    }

    #[test]
    fn test_multiple_scheduler_tasks() {
        let scheduler = create_scheduler();
        let shared_state = create_shared_state();
        let task_count = Arc::new(AtomicBool::new(false));
        let task_count_clone = task_count.clone();

        // Spawn multiple tasks
        let state1 = shared_state.clone();
        scheduler.spawn(move || {
            let mut transport = state1.lock().unwrap();
            transport.set_tempo(120.0);
        });

        let state2 = shared_state.clone();
        scheduler.spawn(move || {
            let mut transport = state2.lock().unwrap();
            transport.set_playing(true);
            task_count_clone.store(true, Ordering::SeqCst);
        });

        // Give tasks time to complete
        thread::sleep(Duration::from_millis(50));

        assert!(
            task_count.load(Ordering::SeqCst),
            "All tasks should have completed"
        );

        // Verify final state
        let transport = shared_state.lock().unwrap();
        assert_eq!(transport.tempo(), 120.0);
        assert!(transport.is_playing());
    }
}
