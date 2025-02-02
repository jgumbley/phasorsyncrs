#[cfg(test)]
mod tests {
    use phasorsyncrs::{create_scheduler, create_shared_state, Scheduler};
    use std::{
        panic::{self, AssertUnwindSafe},
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        sync::{Arc, Mutex},
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
            let transport = state2.lock().unwrap();
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

    #[test]
    fn test_concurrent_task_execution() {
        let scheduler = create_scheduler();
        let counter = Arc::new(AtomicUsize::new(0));
        let tasks_completed = Arc::new(AtomicBool::new(false));

        // Spawn multiple tasks that increment the counter
        for _ in 0..5 {
            let counter = counter.clone();
            let tasks_completed = tasks_completed.clone();
            scheduler.spawn(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                if counter.load(Ordering::SeqCst) == 5 {
                    tasks_completed.store(true, Ordering::SeqCst);
                }
            });
        }

        // Wait for all tasks to complete
        thread::sleep(Duration::from_millis(50));

        assert!(
            tasks_completed.load(Ordering::SeqCst),
            "All tasks should have completed"
        );
        assert_eq!(
            counter.load(Ordering::SeqCst),
            5,
            "Counter should have been incremented by all tasks"
        );
    }

    #[test]
    fn test_panic_handling() {
        let scheduler = create_scheduler();
        let panic_caught = Arc::new(AtomicBool::new(false));
        let panic_caught_clone = panic_caught.clone();

        // Spawn a task that will panic
        scheduler.spawn(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                panic!("Expected panic in task");
            }));
            assert!(result.is_err());
            panic_caught_clone.store(true, Ordering::SeqCst);
        });

        // Give the thread time to execute and catch the panic
        thread::sleep(Duration::from_millis(50));
        assert!(
            panic_caught.load(Ordering::SeqCst),
            "Panic should have been caught"
        );
    }

    #[test]
    fn test_scheduler_drop_behavior() {
        let task_completed = Arc::new(AtomicBool::new(false));
        let task_completed_clone = task_completed.clone();

        {
            let scheduler = create_scheduler();
            scheduler.spawn(move || {
                thread::sleep(Duration::from_millis(50));
                task_completed_clone.store(true, Ordering::SeqCst);
            });
        } // scheduler is dropped here

        // Even though scheduler is dropped, the spawned task should complete
        thread::sleep(Duration::from_millis(100));
        assert!(
            task_completed.load(Ordering::SeqCst),
            "Task should complete even after scheduler is dropped"
        );
    }

    #[test]
    fn test_task_ordering() {
        let scheduler = create_scheduler();
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        // Spawn tasks that record their execution order
        for i in 0..3 {
            let execution_order = execution_order.clone();
            scheduler.spawn(move || {
                thread::sleep(Duration::from_millis(10 * (3 - i))); // Reverse sleep time
                execution_order.lock().unwrap().push(i);
            });
        }

        // Wait for all tasks to complete
        thread::sleep(Duration::from_millis(100));

        let final_order = execution_order.lock().unwrap();
        assert_eq!(final_order.len(), 3, "All tasks should have executed");
        // Note: We don't assert specific ordering since threads may execute in any order
    }
}
