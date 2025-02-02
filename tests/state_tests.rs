use phasorsyncrs::state::TransportState;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_initialization() {
        let state = TransportState::new();
        assert_eq!(state.tempo(), 120.0);
        assert_eq!(state.get_tick_count(), 0);
        assert_eq!(state.get_beat(), 1);
        assert_eq!(state.get_bar(), 1);
        assert!(!state.is_playing());
    }

    #[test]
    fn test_tempo_management() {
        let mut state = TransportState::new();

        // Test setting tempo
        state.set_tempo(140.0);
        assert_eq!(state.tempo(), 140.0);

        // Test extreme tempos
        state.set_tempo(30.0);
        assert_eq!(state.tempo(), 30.0);

        state.set_tempo(300.0);
        assert_eq!(state.tempo(), 300.0);
    }

    #[test]
    fn test_play_state() {
        let state = TransportState::new();

        // Test initial state
        assert!(!state.is_playing());

        // Test play/pause
        state.set_playing(true);
        assert!(state.is_playing());

        state.set_playing(false);
        assert!(!state.is_playing());
    }

    #[test]
    fn test_tick_counting() {
        let state = TransportState::new();

        // Should not tick when not playing
        state.tick();
        assert_eq!(state.get_tick_count(), 0);

        // Should tick when playing
        state.set_playing(true);
        for _ in 0..10 {
            state.tick();
        }
        assert_eq!(state.get_tick_count(), 10);
    }

    #[test]
    fn test_beat_counting() {
        let state = TransportState::new();
        state.set_playing(true);

        // 23 ticks should not change beat
        for _ in 0..23 {
            state.tick();
        }
        assert_eq!(state.get_beat(), 1);

        // 24th tick should increment beat
        state.tick();
        assert_eq!(state.get_beat(), 2);
    }

    #[test]
    fn test_bar_counting() {
        let state = TransportState::new();
        state.set_playing(true);

        // Tick through one full bar (4 beats * 24 ticks = 96 ticks)
        for _ in 0..95 {
            state.tick();
        }
        assert_eq!(state.get_bar(), 1);
        assert_eq!(state.get_beat(), 4);

        // Next tick should increment bar and reset beat
        state.tick();
        assert_eq!(state.get_bar(), 2);
        assert_eq!(state.get_beat(), 1);
    }

    #[test]
    fn test_concurrent_access() {
        let state = Arc::new(TransportState::new());
        state.set_playing(true);

        let mut handles = vec![];

        // Spawn multiple threads to tick simultaneously
        for _ in 0..10 {
            let state_clone = state.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    state_clone.tick();
                    thread::sleep(Duration::from_micros(1));
                }
            }));
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Total ticks should be 1000 (10 threads * 100 ticks)
        assert_eq!(state.get_tick_count(), 1000);
    }

    #[test]
    fn test_long_running_sequence() {
        let state = TransportState::new();
        state.set_playing(true);

        // Run through multiple bars
        for _ in 0..1000 {
            state.tick();
        }

        // Verify counts are consistent
        let tick_count = state.get_tick_count();
        let beat = state.get_beat();
        let bar = state.get_bar();

        // Calculate expected values
        let expected_beats_passed = tick_count / 24;
        let expected_bars_passed = expected_beats_passed / 4;
        let expected_current_beat = ((expected_beats_passed % 4) + 1) as u32;
        let expected_current_bar = (expected_bars_passed + 1) as u32;

        assert_eq!(beat, expected_current_beat);
        assert_eq!(bar, expected_current_bar);
    }

    #[test]
    fn test_rapid_play_pause() {
        let state = Arc::new(TransportState::new());
        let state_clone = state.clone();

        // Spawn a thread that rapidly toggles play state
        let toggle_handle = thread::spawn(move || {
            for _ in 0..100 {
                state_clone.set_playing(true);
                state_clone.set_playing(false);
            }
        });

        // Meanwhile, try to tick in the main thread
        for _ in 0..100 {
            state.tick();
            thread::sleep(Duration::from_micros(1));
        }

        toggle_handle.join().unwrap();

        // Verify the state is still consistent
        assert!(!state.is_playing());
    }

    #[test]
    fn test_boundary_conditions() {
        let state = TransportState::new();
        state.set_playing(true);

        // Test beat transition
        for _ in 0..23 {
            state.tick();
        }
        assert_eq!(state.get_beat(), 1);
        state.tick();
        assert_eq!(state.get_beat(), 2);

        // Test bar transition (complete the remaining 3 beats)
        for _ in 0..72 {
            state.tick();
        }
        assert_eq!(state.get_bar(), 2);
        assert_eq!(state.get_beat(), 1);
    }
}
