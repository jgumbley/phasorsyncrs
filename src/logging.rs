use simplelog::*;
use std::fs::{self, OpenOptions};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();
static mut LOGGER_INITIALIZED: bool = false;

pub fn init_logger() -> Result<(), Error> {
    // Get user's home directory and construct log path
    let home = std::env::var("HOME")
        .map_err(|_| Error::new(ErrorKind::NotFound, "HOME environment variable not set"))?;

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

    // Create config with module path logging enabled
    let config = Config::default();

    // Safe to use unsafe here as we're using Once to ensure single initialization
    unsafe {
        INIT.call_once(|| {
            if let Ok(()) =
                CombinedLogger::init(vec![WriteLogger::new(LevelFilter::Debug, config, log_file)])
            {
                LOGGER_INITIALIZED = true;
            }
        });

        if LOGGER_INITIALIZED {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Logger initialization failed"))
        }
    }
}
