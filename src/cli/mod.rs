use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// List available MIDI devices
    #[arg(long)]
    pub device_list: bool,

    /// Bind to a specific MIDI device
    #[arg(long)]
    pub bind_to_device: Option<String>,
}

pub fn handle_device_list() -> Vec<String> {
    // Re-export from the crate root
    crate::handle_device_list()
}

pub fn validate_device(device_name: &str, devices: &[String]) -> Result<(), String> {
    if !devices.iter().any(|d| d.contains(device_name)) {
        let mut error_msg = format!(
            "Error: Device '{}' not found in available devices:\n",
            device_name
        );
        for device in devices {
            error_msg.push_str(&format!("  - {}\n", device));
        }
        return Err(error_msg);
    }
    Ok(())
}
