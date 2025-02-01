#[cfg(test)]
mod tests {
    use phasorsyncrs::*;

    #[test]
    fn test_device_list() {
        let devices = handle_device_list();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0], "Mock Device 1");
        assert_eq!(devices[1], "Mock Device 2");
    }
}
