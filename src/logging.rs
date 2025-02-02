use simplelog::*;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;

pub fn init_logger() -> Result<(), std::io::Error> {
    // Get user's home directory and construct log path
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let log_dir = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("phasorsyncrs")
        .join("logs");

    // Create the log directory if it doesn't exist
    fs::create_dir_all(&log_dir)?;

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("app.log"))?;

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )])
    .expect("Failed to initialize logger");

    Ok(())
}
