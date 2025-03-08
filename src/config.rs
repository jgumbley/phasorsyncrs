// config.rs

use clap::{Arg, Command};
use log::{debug, info};

pub struct Config {
    pub bpm: u32,
    #[allow(dead_code)]
    pub clock_source: ClockSource,
    #[allow(dead_code)]
    pub default_phasor_length: Option<u32>,
    pub bind_to_device: Option<String>,     // MIDI input device
    pub midi_output_device: Option<String>, // New field for MIDI output
    pub send_test_note: bool,               // For testing MIDI output
    pub direct_test: bool,                  // For direct MIDI output test
}

#[derive(PartialEq)]
pub enum ClockSource {
    Internal,
    External,
}

impl Config {
    fn parse_arguments() -> clap::ArgMatches {
        Command::new("Phasorsyncrs")
            .arg(
                Arg::new("bpm")
                    .short('b')
                    .long("bpm")
                    .value_name("BPM")
                    .help("Sets the beats per minute")
                    .required(false),
            )
            .arg(
                Arg::new("clock-source")
                    .short('c')
                    .long("clock-source")
                    .value_name("SOURCE")
                    .help("Sets the clock source (internal/external)")
                    .required(false),
            )
            .arg(
                Arg::new("bind-to-device")
                    .long("bind-to-device")
                    .value_name("DEVICE")
                    .help("Sets the external MIDI device to bind to")
                    .required(false),
            )
            .arg(
                Arg::new("midi-output")
                    .long("midi-output")
                    .value_name("DEVICE")
                    .help("Sets the MIDI output device")
                    .required(false),
            )
            .arg(
                Arg::new("test-note")
                    .long("test-note")
                    .help("Send a test MIDI note on startup")
                    .action(clap::ArgAction::SetTrue)
                    .required(false),
            )
            .arg(
                Arg::new("direct-test")
                    .long("direct-test")
                    .help("Run a direct MIDI output test")
                    .action(clap::ArgAction::SetTrue)
                    .required(false),
            )
            .get_matches()
    }

    // Parse BPM from command line arguments
    fn parse_bpm(matches: &clap::ArgMatches) -> u32 {
        let bpm = matches
            .get_one::<String>("bpm")
            .map(|s| s.as_str())
            .unwrap_or("120")
            .parse::<u32>()
            .unwrap_or(120);

        debug!("Parsed BPM value: {}", bpm);
        bpm
    }

    // Determine clock source based on arguments
    fn determine_clock_source(matches: &clap::ArgMatches) -> ClockSource {
        let clock_source_arg = matches
            .get_one::<String>("clock-source")
            .map(|s| s.as_str())
            .unwrap_or("internal");

        debug!("Raw clock-source argument: {:?}", clock_source_arg);

        let bind_to_device = matches.get_one::<String>("bind-to-device").is_some();

        if bind_to_device {
            info!("External device specified, forcing external clock mode");
            ClockSource::External
        } else if clock_source_arg == "external" {
            info!("External clock mode selected via --clock-source");
            ClockSource::External
        } else {
            info!("Using internal clock mode");
            ClockSource::Internal
        }
    }

    pub fn new() -> Self {
        let matches = Self::parse_arguments();

        // Parse BPM
        let bpm = Self::parse_bpm(&matches);

        // Get bind-to-device
        let bind_to_device = matches.get_one::<String>("bind-to-device").cloned();
        debug!("Bind-to-device argument: {:?}", bind_to_device);

        // Determine clock source
        let clock_source = Self::determine_clock_source(&matches);

        // New MIDI output device option
        let midi_output_device = matches.get_one::<String>("midi-output").cloned();
        debug!("MIDI output device argument: {:?}", midi_output_device);

        // Test note flag
        let send_test_note = matches.get_flag("test-note");
        if send_test_note {
            info!("Test note flag enabled - will send a test note on startup");
        }

        // Direct test flag
        let direct_test = matches.get_flag("direct-test");
        if direct_test {
            info!("Direct MIDI test flag enabled - will run direct MIDI output test");
        }

        Config {
            bpm,
            clock_source,
            default_phasor_length: None,
            bind_to_device,
            midi_output_device,
            send_test_note,
            direct_test,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub const TICKS_PER_BEAT: u64 = 24;
pub const BEATS_PER_BAR: u64 = 4;
pub const BARS_PER_PHRASE: u64 = 4;
