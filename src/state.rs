// state.rs

pub struct SharedState {
    bpm: u32,
    tick_count: u64,
    current_beat: u32,
    current_bar: u32,
}

impl SharedState {
    pub fn new(bpm: u32) -> Self {
        SharedState {
            bpm,
            tick_count: 0,
            current_beat: 0,
            current_bar: 0,
        }
    }

    pub fn tick_update(&mut self) {
        self.tick_count += 1;
        self.current_beat = (self.tick_count % (self.bpm as u64)) as u32;
        self.current_bar = self.current_beat / 4; // Assuming 4 beats per bar
    }

    pub fn get_bpm(&self) -> u32 {
        self.bpm
    }

    pub fn get_tick_count(&self) -> u64 {
        self.tick_count
    }

    pub fn get_current_beat(&self) -> u32 {
        self.current_beat
    }

    pub fn get_current_bar(&self) -> u32 {
        self.current_bar
    }
}
