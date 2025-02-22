# ADR 03: Structure, Concurrency and Instantiation

## Context

The project is a command line real-time MIDI sequencer engine. Its core functionality includes:

- Syncing to an external MIDI clock (from devices such as the OXI).
- Recording incoming MIDI events into a cycle structure defined in musical terms (beats and bars).
- Supporting multiple pattern providers that generate harmonizing MIDI patterns aligned with the cycle structure.
- Scheduling and playing back MIDI events with precise timing.

In addition to the core engine, the application must provide a command line monitoring interface that displays the current BPM, transport state (started, stopped, paused), cycle position, pattern status, and other diagnostic information. The design must support incremental development and modularity.

## Decision

We will adopt a multi-threaded architecture that separates the real-time processing of MIDI events from the command line status inspection, using message passing as the primary communication mechanism. The key design decisions are as follows:

- **Central Shared State:**
  A single shared state object will hold key parameters, including:
  - Current BPM (tempo)
  - Transport status (e.g., Running, Stopped, Paused)
  - Current cycle information (current bar, beat, tick)
  - Pattern recording and playback status
  - Any additional diagnostic metrics

  This state will be managed using a mutex and will only be directly accessed by the tick processing thread.

- **Message Passing System:**
  A `std::sync::mpsc` channel will serve as the primary communication mechanism between components:
  ```rust
  enum Message {
      MIDITick,
      UIUpdate(StateSnapshot),
      TransportCommand(TransportAction),
      // Extensible for future message types
  }
  ```
  This design ensures that all inter-thread communication happens through explicit message passing rather than shared memory access.

- **Core Engine Threads:**
  The system will use several dedicated threads with specific responsibilities:
  - **MIDI Callback Thread:** Receives external MIDI clock signals and immediately forwards them as `MIDITick` messages through the channel
  - **Connection Maintenance Thread:** Ensures MIDI connectivity remains stable
  - **Tick Processing Thread:**
    - Receives messages from the channel
    - Updates the shared state based on MIDI ticks
    - Generates UI update messages containing state snapshots
    - Handles scheduling and pattern processing

- **Inspector Thread:**
  The inspector thread will:
  - Receive UI update messages from the channel containing state snapshots
  - Display real-time status information on the command line
  - Never directly access the shared state, preventing UI-related mutex contention

- **Instantiation and Thread Management:**
  The main application will:
  - Initialize the shared state with default values
  - Create the message passing channel
  - Spawn the MIDI callback thread and tick processing thread
  - Spawn the inspector thread to receive and display UI updates
  - Implement graceful shutdown through channel-based signaling

## Consequences

- **Improved Real-time Performance:**
  - MIDI callback remains extremely lightweight, only sending a message
  - No mutex contention from UI updates
  - Natural backpressure handling through the channel system

- **Simplified Concurrency Model:**
  - Single writer (tick processing thread) to shared state
  - All other communication happens through message passing
  - Eliminates potential deadlocks from UI-related mutex access
  - Clear ownership and responsibility boundaries

- **Enhanced Modularity:**
  - Components are fully decoupled through message passing
  - New features can be added by extending the Message enum
  - UI updates can be throttled or batched without affecting MIDI processing
  - Easy to add new UI consumers without changing core logic

- **Debugging and Testing:**
  - Message passing makes it easier to trace system behavior
  - Can log or inspect all inter-thread communication
  - Easier to test components in isolation
  - Can simulate different timing scenarios by manipulating message flow

## Alternatives Considered

- **Direct Shared State Access:**
  Rejected due to mutex contention issues and potential for UI updates to block MIDI processing.

- **Multiple Channels:**
  Considered using separate channels for MIDI events and UI updates. While this could prevent UI updates from queueing behind MIDI ticks, the added complexity and potential for message reordering made a single channel preferable for the initial implementation.

- **Lock-free Data Structures:**
  While lock-free approaches could potentially improve performance, the simplicity and proven reliability of the mpsc channel makes it a better choice for the initial implementation.

## Status

Proposed
