// config.rs

use clap::{Arg, Command};

pub struct Config {
    pub bpm: u32,
    #[allow(dead_code)]
    pub clock_source: ClockSource,
    #[allow(dead_code)]
    pub default_phasor_length: Option<u32>,
}

pub enum ClockSource {
    Internal,
    External,
}

impl Config {
    pub fn new() -> Self {
        let matches = Command::new("Phasorsyncrs")
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
            .get_matches();

        let bpm = matches
            .get_one::<String>("bpm")
            .map(|s| s.as_str())
            .unwrap_or("120")
            .parse::<u32>()
            .unwrap_or(120);

        let clock_source = match matches
            .get_one::<String>("clock-source")
            .map(|s| s.as_str())
            .unwrap_or("internal")
        {
            "external" => ClockSource::External,
            _ => ClockSource::Internal,
        };

        Config {
            bpm,
            clock_source,
            default_phasor_length: None,
        }
    }
}

pub const TICKS_PER_BEAT: u64 = 24;
pub const BEATS_PER_BAR: u64 = 4;
pub const BARS_PER_PHRASE: u64 = 4;
