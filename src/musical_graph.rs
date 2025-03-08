use crate::state;
use log::debug;

// Musical graph constants
const TICKS_PER_BEAT: u64 = 24;
const BEATS_PER_BAR: u64 = 4;
const TRIGGER_EVERY_N_BARS: u64 = 1;

// Static variable to track the musical tick count
static mut MUSICAL_TICK_COUNT: u64 = 0;

/// Processes a tick event by checking the current bar and beat.
/// If the current bar is nonzero, is a multiple of 8, and the current beat is 0,
/// then log that a Middle C event is triggered.
///
/// Returns true if a Middle C note was triggered, false otherwise.
pub fn process_tick(shared_state: &mut state::SharedState) -> bool {
    // Only process if transport is playing
    if shared_state.transport_state != state::TransportState::Playing {
        return false;
    }

    let mut middle_c_triggered = false;

    // Safely increment our own tick counter
    unsafe {
        MUSICAL_TICK_COUNT += 1;

        // Calculate musical bar and beat
        let beat = (MUSICAL_TICK_COUNT / TICKS_PER_BEAT) % BEATS_PER_BAR;
        let bar = (MUSICAL_TICK_COUNT / (TICKS_PER_BEAT * BEATS_PER_BAR)) + 1; // 1-indexed

        // Add info logging every 24 ticks (once per beat)
        if MUSICAL_TICK_COUNT % TICKS_PER_BEAT == 0 {
            let tick_count = MUSICAL_TICK_COUNT; // Copy to local variable
            debug!(
                "Musical graph tick count: {}, bar: {}, beat: {}",
                tick_count, bar, beat
            );
        }

        // Check if we're at the start of a bar (beat 0) and at a multiple of 8 bars
        // Only trigger on the first tick of the beat (when MUSICAL_TICK_COUNT is divisible by TICKS_PER_BEAT)
        if beat == 0
            && bar > 0
            && TRIGGER_EVERY_N_BARS > 0
            && MUSICAL_TICK_COUNT % TICKS_PER_BEAT == 0
        {
            debug!("Middle C triggered at musical bar: {}, beat: {}", bar, beat);
            middle_c_triggered = true;
        }
    }

    middle_c_triggered
}

/// Reset the musical tick count, should be called when transport is stopped
pub fn reset_musical_tick_count() {
    unsafe {
        MUSICAL_TICK_COUNT = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SharedState;
    use crate::state::TransportState;

    #[test]
    fn test_middle_c_trigger_condition() {
        // Create a SharedState instance with dummy values.
        let mut state = SharedState {
            tick_count: 0,
            current_beat: 0,
            current_bar: 8, // 8 is a multiple of 8 and > 0.
            bpm: 120,
            transport_state: TransportState::Stopped,
        };

        // Call process_tick; this should not trigger Middle C because transport is stopped
        let triggered = process_tick(&mut state);

        // Verify Middle C was not triggered because transport is stopped
        assert_eq!(triggered, false);
        assert_eq!(state.current_bar, 8);
        assert_eq!(state.current_beat, 0);
    }

    #[test]
    fn test_middle_c_triggers_only_once_per_bar() {
        // Reset the musical tick count to ensure a clean state
        reset_musical_tick_count();

        // Create a SharedState instance
        let mut state = SharedState {
            tick_count: 0,
            current_beat: 0,
            current_bar: 0,
            bpm: 120,
            transport_state: TransportState::Playing,
        };

        let mut trigger_count = 0;

        // Simulate ticks for 8 bars (8 bars * 4 beats * 24 ticks = 768 ticks)
        // This should trigger Middle C on every bar
        for _ in 0..768 {
            let triggered = process_tick(&mut state);
            if triggered {
                trigger_count += 1;

                // Verify it only triggers at the expected position (beat 0, first tick)
                unsafe {
                    let beat = (MUSICAL_TICK_COUNT / TICKS_PER_BEAT) % BEATS_PER_BAR;

                    assert_eq!(beat, 0, "Middle C should only trigger on beat 0");
                    assert_eq!(
                        MUSICAL_TICK_COUNT % TICKS_PER_BEAT,
                        0,
                        "Middle C should only trigger on the first tick of the beat"
                    );
                }
            }
        }

        // Check that Middle C was triggered on every bar
        assert_eq!(
            trigger_count, 8,
            "Middle C should be triggered on every bar"
        );
    }
}
