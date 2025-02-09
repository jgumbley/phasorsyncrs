# MIDI Engine Separation Refactor (v0.2.1 Hotfix)

## Critical Corrections
```diff
- [WRONG] Modifying MidirEngine (external MIDI)
+ [FIXED] Create new InternalEngine for clock generation
```

## Implementation Plan

### 1. Engine Creation (30min)
```rust
// src/midi/internal_engine.rs
+pub struct InternalEngine {
+    core: Arc<Mutex<ClockCore>>,
+}

+impl MidiEngine for InternalEngine {
+    fn send(&mut self, msg: MidiMessage) -> Result<()> {
+        self.core.lock().unwrap().process_message(msg);
+        Ok(())
+    }
+}
```

### 2. Thread Migration (1h)
```rust
// src/midi/internal_clock.rs (BEFORE)
- thread::spawn(move || {
-     while running.load(Ordering::Relaxed) {
-         // Clock generation logic
-     }
- });

// src/midi/internal_engine.rs (AFTER)
+impl InternalEngine {
+    pub fn start(&self) -> JoinHandle<()> {
+        let core = self.core.clone();
+        thread::spawn(move || {
+            while core.lock().unwrap().is_running() {
+                // Clock generation logic
+            }
+        })
+    }
+}
```

### 3. Interface Updates (30min)
```rust
// src/midi/mod.rs
+pub enum MidiEngineType {
+    Internal(InternalEngine),
+    External(MidirEngine),
+    Mock(MockEngine),
+}
```

## Validation Checklist
```bash
# Confirm thread moved to internal_engine
rg "thread::spawn" src/midi/internal_engine.rs

# Verify engine separation
cargo check --features midir
cargo test --test midi_engine_tests
```

## Rollback Plan
```bash
git checkout HEAD -- src/midi/internal_clock.rs && \
rm src/midi/internal_engine.rs
```

## Architectural Diagram
```
MIDI Engines
├── InternalEngine (clock generation)
├── MidirEngine (external devices)
└── MockEngine (testing)

InternalClock
└── Uses InternalEngine