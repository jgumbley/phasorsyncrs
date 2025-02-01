//! Module for handling MIDI device interactions

#[cfg(not(feature = "test-mock"))]
pub fn list_devices() -> Vec<String> {
    let seq = match alsa::Seq::open(None, None, false) {
        Ok(s) => s,
        Err(_) => return vec![], // Return empty list if we can't open sequencer
    };

    let mut devices = Vec::new();

    // Create client iterator and iterate through clients
    let client_iter = alsa::seq::ClientIter::new(&seq);

    for client_info in client_iter {
        let client_id = client_info.get_client();
        let client_name = client_info.get_name().unwrap_or_default();

        // Create port iterator and iterate through ports
        let port_iter = alsa::seq::PortIter::new(&seq, client_id);

        for port_info in port_iter {
            let port_id = port_info.get_port();

            // Create address for port info lookup
            let addr = alsa::seq::Addr {
                client: client_id,
                port: port_id,
            };

            if let Ok(port_info) = seq.get_any_port_info(addr) {
                if let Ok(port_name) = port_info.get_name() {
                    let caps = port_info.get_capability();
                    let mut capabilities = Vec::new();

                    if caps.contains(alsa::seq::PortCap::READ) {
                        capabilities.push("Input");
                    }
                    if caps.contains(alsa::seq::PortCap::WRITE) {
                        capabilities.push("Output");
                    }

                    if !capabilities.is_empty() {
                        devices.push(format!(
                            "{} - {} ({}:{}) [{}]",
                            client_name,
                            port_name,
                            client_id,
                            port_id,
                            capabilities.join("/")
                        ));
                    }
                }
            }
        }
    }

    devices
}

#[cfg(feature = "test-mock")]
pub fn list_devices() -> Vec<String> {
    // Mock implementation for tests - simple format as expected by tests
    vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()]
}
