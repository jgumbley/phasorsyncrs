use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};

pub struct TransportState {
    bpm: f64, // TODO: Replace with AtomicF64 once available
    pub tick_count: AtomicU64,
    pub beat: AtomicU32,
    pub bar: AtomicU32,
    pub is_playing: AtomicBool,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            bpm: 120.0, // Default BPM
            tick_count: AtomicU64::new(0),
            beat: AtomicU32::new(1),
            bar: AtomicU32::new(1),
            is_playing: AtomicBool::new(false),
        }
    }
}

impl TransportState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tempo(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    pub fn tempo(&self) -> f64 {
        self.bpm
    }

    pub fn set_playing(&self, playing: bool) {
        self.is_playing
            .store(playing, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn tick(&self) {
        if !self.is_playing() {
            return;
        }

        let new_tick = self
            .tick_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Update beat and bar
        if (new_tick + 1) % 24 == 0 {
            // TICKS_PER_BEAT = 24
            let current_beat = self.beat.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // If we've reached the end of a bar (assuming 4/4 time signature)
            if current_beat + 1 > 4 {
                self.beat.store(1, std::sync::atomic::Ordering::SeqCst);
                self.bar.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    pub fn get_tick_count(&self) -> u64 {
        self.tick_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn get_beat(&self) -> u32 {
        self.beat.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn get_bar(&self) -> u32 {
        self.bar.load(std::sync::atomic::Ordering::SeqCst)
    }
}
