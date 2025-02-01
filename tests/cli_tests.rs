#[cfg(test)]
mod tests {
    use phasorsyncrs::*;

    #[test]
    fn test_get_user_name_with_args() {
        let test_name = String::from("TestUser");
        let result = get_user_name(Some(test_name.clone()));
        assert_eq!(result, test_name);
    }

    #[test]
    fn test_device_list() {
        let devices = handle_device_list();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0], "Mock Device 1");
        assert_eq!(devices[1], "Mock Device 2");
    }
}
