// config.rs

use clap::{Arg, Command};
use log::{debug, info};

pub struct Config {
    pub bpm: u32,
    #[allow(dead_code)]
    pub clock_source: ClockSource,
    #[allow(dead_code)]
    pub default_phasor_length: Option<u32>,
    pub bind_to_device: Option<String>, // New field for external sync
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
            .get_matches()
    }

    pub fn new() -> Self {
        let matches = Self::parse_arguments();

        let bpm = matches
            .get_one::<String>("bpm")
            .map(|s| s.as_str())
            .unwrap_or("120")
            .parse::<u32>()
            .unwrap_or(120);

        debug!("Parsed BPM value: {}", bpm);

        let clock_source_arg = matches
            .get_one::<String>("clock-source")
            .map(|s| s.as_str())
            .unwrap_or("internal");

        debug!("Raw clock-source argument: {:?}", clock_source_arg);

        let bind_to_device = matches.get_one::<String>("bind-to-device").cloned();
        debug!("Bind-to-device argument: {:?}", bind_to_device);

        // Modify clock source selection logic
        let clock_source = if bind_to_device.is_some() {
            info!("External device specified, forcing external clock mode");
            ClockSource::External
        } else {
            match clock_source_arg {
                "external" => {
                    info!("External clock mode selected via --clock-source");
                    ClockSource::External
                }
                _ => {
                    info!("Using internal clock mode");
                    ClockSource::Internal
                }
            }
        };

        Config {
            bpm,
            clock_source,
            default_phasor_length: None,
            bind_to_device,
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
