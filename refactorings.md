# PhasorSyncRS Architectural Refactorings

## Urgent Priority

### 2. MIDI Abstraction Layer Missing (ADR01/ADR02)
**Files:** src/main.rs, src/midi/midir_engine.rs  
**Instructions:**
- Create `src/midi/mod.rs` with:
  ```rust
  pub trait MidiEngine: Send {
      fn send(&amp;mut self, msg: MidiMessage) -> Result<(), MidiError>;
      fn recv(&amp;self) -> Result<MidiMessage, MidiError>;
  }
  ```
- Update MidirEngine to implement trait
- Remove direct MidirEngine references from main.rs

## High Priority

### 3. Shared State Management (ADR03 Violation)
**Files:** src/main.rs, src/lib.rs  
**Instructions:**
- Implement atomic reference counting instead of mutex cloning
- Create state module with:
  ```rust
  pub struct TransportState {
      bpm: AtomicF64,
      tick_count: AtomicU64,
      // ... other fields
  }
  ```

### 4. Test Coverage Required (ADR01)
**Files:** All production modules  
**Instructions:**
- Add unit tests adjacent to each module:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn transport_ticks_correctly() {
          // Test tick increments
      }
  }
  ```
- Create `tests/integration/` directory for hardware tests

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