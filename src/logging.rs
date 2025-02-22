use simplelog::*;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();
static mut LOGGER_INITIALIZED: bool = false;

pub fn init_logger() -> Result<(), Error> {
    // Construct log path in the current directory
    let log_file_path = PathBuf::from("app.log");

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

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
