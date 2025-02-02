#[cfg(test)]
mod tests {
    use clap::Parser;
    use phasorsyncrs::cli::{validate_device, Args};
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
    fn test_args_with_device_list() {
        let args = Args::parse_from(["test", "--device-list"]);
        assert!(args.device_list);
        assert_eq!(args.bind_to_device, None);
    }

    #[test]
    fn test_args_with_both_options() {
        let args = Args::parse_from(["test", "--device-list", "--bind-to-device", "Mock Device 1"]);
        assert!(args.device_list);
        assert_eq!(args.bind_to_device, Some("Mock Device 1".to_string()));
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

    #[test]
    fn test_device_validation_success() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("Mock Device 1", &devices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_validation_failure() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("Nonexistent Device", &devices);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Error: Device 'Nonexistent Device' not found"));
        assert!(error.contains("Mock Device 1"));
        assert!(error.contains("Mock Device 2"));
    }

    #[test]
    fn test_device_validation_partial_match() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("Mock", &devices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_validation_case_sensitive() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("mock device 1", &devices);
        assert!(
            result.is_err(),
            "Device validation should be case sensitive"
        );
    }

    #[test]
    fn test_device_validation_empty_list() {
        let devices: Vec<String> = vec![];
        let result = validate_device("Any Device", &devices);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Error: Device 'Any Device' not found"));
    }

    #[test]
    fn test_device_validation_empty_name() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("", &devices);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Error: Device '' not found"));
    }

    #[test]
    fn test_device_validation_whitespace_name() {
        let devices = vec!["Mock Device 1".to_string(), "Mock Device 2".to_string()];
        let result = validate_device("   ", &devices);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("Error: Device '   ' not found"));
    }

    #[test]
    fn test_device_list_flag() {
        let args = Args::parse_from(["test", "--device-list"]);
        assert!(args.device_list);
    }

    #[test]
    fn test_device_list_short_flag() {
        let result = Args::try_parse_from(["test", "-l"]);
        assert!(result.is_err(), "Short flag -l should not be accepted");
    }
}
