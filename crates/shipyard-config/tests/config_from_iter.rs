use shipyard_config::{AppConfig, Environment};

#[test]
fn defaults_load_when_env_is_empty() {
    let cfg = AppConfig::from_kv(std::iter::empty::<(&str, &str)>()).unwrap();
    assert_eq!(cfg.env, Environment::Dev);
    assert_eq!(cfg.service_port, 8080);
}

#[test]
fn invalid_port_fails_fast() {
    let err = AppConfig::from_kv([("SERVICE_PORT", "0")]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("SERVICE_PORT"));
}
