# Music Theory Layer

This document outlines the planned architecture for a music theory layer in Rust, inspired by Python's music21 library.

## MIDI Layer

The foundation for MIDI file handling will be the `midly` crate.

## Music Theory Primitives

Custom Rust types will be created to represent musical concepts.

## TinyNotation Parser

A custom parser for a subset of TinyNotation will be implemented to create musical patterns directly from string representations.

## Gaps to Address

1.  **Rhythm Calculations**: Custom implementation needed for complex rhythmic calculations.
2.  **Chord Detection**: Requires implementation of music theory rules for chord recognition.
3.  **Key/Scale Analysis**: Missing from current Rust crates, needs custom implementation.

## Implementation Roadmap

1.  **Phase 1**: Implement basic Note and Stream types.
2.  **Phase 2**: Develop TinyNotation parser.
3.  **Phase 3**: Add chord and scale analysis.
4.  **Phase 4**: Integrate with MIDI layer.

## Performance Considerations

Rust's performance capabilities make it ideal for real-time MIDI processing.