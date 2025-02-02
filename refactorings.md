# PhasorSyncRS Architectural Refactorings


## Medium Priority

### 5. Event-Driven Architecture (ADR03)
**Files:** src/main.rs, src/lib.rs  
**Instructions:**
- Implement crossbeam-channel for thread communication
- Replace shared state cloning with event passing:
  ```rust
  enum TransportEvent {
      Tick(u64),
      BeatChange(u8),
      BarChange(u16),
  }
  ```

### 6. Configuration Hardcoding
**Files:** src/main.rs  
**Instructions:
- Extract magic numbers to config struct
- Create `src/config.rs` with:
  ```rust
  pub struct MidiConfig {
      pub ticks_per_beat: u32,
      pub time_signature: (u8, u8),
  }