# Music Theory Layer for Rust MIDI Implementation

This document outlines the recommended architecture and implementation strategy for a Rust-based music theory layer that replicates and enhances the functionality provided by Python's music21 library in the current Midimoke implementation.

## 1. MIDI Layer

We recommend using the `midly` crate (version 0.5.1 or higher) as the foundation for MIDI file handling. This crate provides robust support for reading and writing MIDI files, events, and messages.

### Example Implementation:
```rust
use midly::{MidiFile, TrackEvent, Event::*};

struct MidiEvent {
    delta: u32,
    message: TrackEvent,
}

impl From<Note> for MidiEvent {
    fn from(note: Note) -> Self {
        Self {
            delta: 0,
            message: TrackEvent::NoteOn {
                channel: 0,
                note: note.pitch,
                velocity: note.velocity,
            },
        }
    }
}
```

## 2. Music Theory Primitives

We will create custom Rust types to represent musical concepts, mirroring the functionality of music21's core objects.

### Note Structure:
```rust
#[derive(Clone, Debug)]
pub struct Note {
    pub pitch: midly::num::u7,
    pub duration: MusicalTime,
    pub velocity: u7,
}

#[derive(Clone, Debug)]
pub struct MusicalTime {
    pub beats: f32,
    pub div_type: DivisionType,
}

pub enum DivisionType {
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}
```

### Stream Abstraction:
```rust
pub struct EventStream {
    pub tempo: u32,
    pub time_sig: (u8, u8),
    pub events: Vec<(MusicalTime, Event)>,
}

impl EventStream {
    pub fn append(&mut self, event: Event, duration: MusicalTime) {
        // Implementation to handle time positioning
        self.events.push((duration, event));
    }
}
```

## 3. TinyNotation Parser

We will implement a custom parser for a subset of TinyNotation to create musical patterns directly from string representations.

### Parser Implementation:
```rust
mod tinynotation {
    pub fn parse(input: &str) -> Result<EventStream> {
        // Implementation to parse patterns like "6/8 e4. d8 c# d e2."
        unimplemented!()
    }
}
```

## 4. Gaps to Address

1. **Rhythm Calculations**: Custom implementation needed for complex rhythmic calculations.
2. **Chord Detection**: Requires implementation of music theory rules for chord recognition.
3. **Key/Scale Analysis**: Missing from current Rust crates, needs custom implementation.

## 5. Implementation Roadmap

1. **Phase 1**: Implement basic Note and Stream types.
2. **Phase 2**: Develop TinyNotation parser.
3. **Phase 3**: Add chord and scale analysis.
4. **Phase 4**: Integrate with MIDI layer.

## 6. Performance Considerations

Rust's performance capabilities make it ideal for real-time MIDI processing. The recommended architecture ensures:
- Memory safety
- Efficient event handling
- Low-latency processing

## 7. Example Usage

```rust
fn main() {
    let mut stream = EventStream::new();
    let note = Note {
        pitch: 60, // C4
        duration: MusicalTime {
            beats: 1.0,
            div_type: DivisionType::Quarter,
        },
        velocity: 64,
    };
    stream.append(note.into(), note.duration);
}
```

This implementation provides a foundation for building sophisticated musical applications in Rust, maintaining the expressiveness of music21 while leveraging Rust's performance and safety features.