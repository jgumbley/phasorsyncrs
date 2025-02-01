use phasorsyncrs::*;

#[test]
fn test_get_user_name_with_args() {
    let test_name = String::from("TestUser");
    let result = get_user_name(Some(test_name.clone()));
    assert_eq!(result, test_name);
}
