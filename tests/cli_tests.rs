#[cfg(test)]
mod tests {
    use clap::Parser;
    use phasorsyncrs::*;

    #[test]
    fn test_device_list() {
        let devices = handle_device_list();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0], "Mock Device 1");
        assert_eq!(devices[1], "Mock Device 2");
    }

    #[test]
    fn test_args_with_device_binding() {
        let args = Args::parse_from(["test", "--bind-to-device", "Mock Device 1"]);
        assert_eq!(args.bind_to_device, Some("Mock Device 1".to_string()));
        assert!(!args.device_list);
    }

    #[test]
    fn test_args_without_device_binding() {
        let args = Args::parse_from(["test"]);
        assert_eq!(args.bind_to_device, None);
        assert!(!args.device_list);
    }

    #[test]
    fn test_valid_device_binding() {
        let devices = handle_device_list();
        let device_name = "Mock Device 1";
        assert!(
            devices.iter().any(|d| d.contains(device_name)),
            "Valid device '{}' should be found in device list",
            device_name
        );
    }

    #[test]
    fn test_invalid_device_binding() {
        let devices = handle_device_list();
        let device_name = "Nonexistent Device";
        assert!(
            !devices.iter().any(|d| d.contains(device_name)),
            "Invalid device '{}' should not be found in device list",
            device_name
        );
    }
}
