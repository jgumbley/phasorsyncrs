use crate::SharedState;
use std::thread;

pub fn run_midi_input<T>(engine: T, _state: SharedState) -> thread::JoinHandle<()>
where
    T: crate::midi::MidiEngine + Send + 'static,
{
    thread::spawn(move || {
        println!("MIDI input thread started");
        loop {
            match engine.recv() {
                Ok(msg) => {
                    println!("Received MIDI message: {:?}", msg);
                    // TODO: Handle different MIDI message types
                }
                Err(e) => {
                    eprintln!("Error receiving MIDI message: {}", e);
                    break;
                }
            }
        }
    })
}
