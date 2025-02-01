# ADR 03: Structure, Concurrency and Instantiation

## Context

The project is a command line real-time MIDI sequencer engine. Its core functionality includes:

- Syncing to an external MIDI clock (from devices such as the OXI).
- Recording incoming MIDI events into a cycle structure defined in musical terms (beats and bars).
- Supporting multiple pattern providers that generate harmonizing MIDI patterns aligned with the cycle structure.
- Scheduling and playing back MIDI events with precise timing.

In addition to the core engine, the application must provide a command line monitoring interface that displays the current BPM, transport state (started, stopped, paused), cycle position, pattern status, and other diagnostic information. The design must support incremental development and modularity.

## Decision

We will adopt a multi-threaded architecture that separates the real-time processing of MIDI events from the command line status inspection. The key design decisions are as follows:

- **Central Shared State:**  
  A single shared state object will be defined to hold key parameters, including:
  - Current BPM (tempo)
  - Transport status (e.g., Running, Stopped, Paused)
  - Current cycle information (current bar, beat, tick)
  - Pattern recording and playback status
  - Any additional diagnostic metrics

  This state will be managed using a thread-safe mechanism (such as a shared, lock-protected data structure) to allow concurrent access by multiple threads.

- **Core Engine Threads:**  
  One or more dedicated worker threads will be responsible for:
  - Processing the external MIDI clock (e.g., receiving clock pulses and transport messages).
  - Converting MIDI clock ticks into musical timing (beats, bars).
  - Recording incoming MIDI events relative to the cycle boundaries.
  - Querying registered pattern providers and scheduling playback events.

  These threads will update the central shared state as they process incoming events and perform scheduling.

- **Inspector Thread:**  
  A separate inspector thread will run concurrently and periodically read the shared state to display real-time status information on the command line. This thread will:
  - Monitor changes in BPM, transport state, cycle position, and pattern status.
  - Present this information in a human-readable format for diagnostic and monitoring purposes.

- **Instantiation and Thread Management:**  
  The main application will:
  - Initialize the shared state with default values.
  - Spawn the core engine thread(s) to handle MIDI processing and scheduling.
  - Spawn the inspector thread to continuously display the current status.
  - Use well-defined shutdown and inter-thread communication mechanisms (e.g., channels or atomic flags) to ensure graceful termination and coordinated state updates.

## Consequences

- **Modularity:**  
  Separating the core processing (MIDI I/O, timing, event scheduling) from the status monitoring allows each component to evolve independently. Future enhancements (such as audio recording or additional pattern generators) can be integrated without affecting the monitoring layer.

- **Thread Safety and Concurrency:**  
  A centralized shared state managed via a thread-safe construct minimizes the risk of race conditions and simplifies debugging. Real-time processing is isolated in dedicated threads, ensuring that UI or monitoring activities do not interfere with timing-critical operations.

- **Incremental Development:**  
  This architecture enables coding agents to develop the application step-by-step. The core engine can be built and tested independently, after which the inspector thread can be added to provide real-time feedback. Further modules (e.g., for extended MIDI analysis or audio integration) can be plugged in later without reworking the core design.

- **Responsiveness:**  
  By using a dedicated inspector thread, the application can provide continuous, non-blocking status updates on the command line, thereby aiding in debugging and performance tuning.

## Alternatives Considered

- **Single-Threaded Design:**  
  A single-threaded approach was rejected because the real-time MIDI processing and status monitoring have conflicting timing requirements. Blocking the processing loop to update the UI would compromise the precision needed for MIDI scheduling.

- **Asynchronous (async/await) Model:**  
  Although an asynchronous approach was considered, the inherent real-time nature of MIDI scheduling and the need for predictable low-latency behavior favor a multi-threaded model with explicit shared state management.

## Status

Proposed
