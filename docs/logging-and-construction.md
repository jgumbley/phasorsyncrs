# Logging Implementation Strategy

## Core Principles
1. **Decoupled Architecture**: Use Rust's trait system (`Logger: log::Log`) for dependency inversion
2. **Testability**: Mock logger implementation available under `#[cfg(test)]`
3. **Operational Efficiency**: Daily rotated logs in `/var/log/phasorsyncrs/` (create if missing)

## Implementation Priorities

### 1. Add Dependencies (`Cargo.toml`)
```toml
[dependencies]
log = "0.4"
simplelog = { version = "0.12", features = ["termcolor", "humantime"] }
chrono = "0.4"
```

### 2. Logger Configuration (`src/logging.rs`)
```rust
use simplelog::*;
use std::fs::OpenOptions;

pub fn init_logger() -> Result<(), std::io::Error> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/var/log/phasorsyncrs/app.log")?;

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            log_file,
        ),
    ]).expect("Failed to initialize logger");

    Ok(())
}
```

### 3. Application Initialization (`src/main.rs`)
```rust
mod logging;

fn main() {
    logging::init_logger().expect("Logger initialization failed");
    log::info!("Application starting");
    // Existing startup code
}
```

### 4. Usage Pattern
```rust
use log::{info, error};

fn process_midi(&self) {
    info!("Processing MIDI event: {:?}", event);
    // Core logic
    if let Err(e) = result {
        error!("MIDI processing failed: {}", e);
    }
}
```

## Monitoring Commands
```bash
# Follow logs in realtime
tail -f /var/log/phasorsyncrs/app.log | ack --passthru WARNING

# View with syntax highlighting
bat --language log /var/log/phasorsyncrs/app.log
```

## Test Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    struct MockLogger;
    
    impl log::Log for MockLogger {
        fn enabled(&self, _: &log::Metadata) -> bool { true }
        fn log(&self, record: &log::Record) {
            println!("[TEST] {} - {}", record.level(), record.args());
        }
        fn flush(&self) {}
    }

    #[test]
    fn test_logging() {
        let _ = log::set_boxed_logger(Box::new(MockLogger)).unwrap();
        log::set_max_level(LevelFilter::Debug);
    }
}