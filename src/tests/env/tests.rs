use super::*;

#[test]
fn environment_values_are_validated() {
    assert!(Env::from_values(Some("wyndis_api=invalid")).is_err());
    let env = Env::from_values(None).unwrap();
    assert_eq!(env.log_filter.to_string(), "wyndis_api=info");
    let env = Env::from_values(Some("debug")).unwrap();
    assert_eq!(env.log_filter.to_string(), "debug");
}
