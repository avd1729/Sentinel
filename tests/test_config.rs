use sentinel::config::Config;

#[test]
fn test_config_default_address() {
    // When LISTEN env var is not set, should use default
    unsafe {
        std::env::remove_var("LISTEN");
    }
    let cfg = Config::load();
    assert_eq!(cfg.listen_addr, "127.0.0.1:8080");
}

#[test]
fn test_config_custom_address_from_env() {
    // When LISTEN env var is set, should use it
    unsafe {
        std::env::set_var("LISTEN", "0.0.0.0:3000");
    }
    let cfg = Config::load();
    assert_eq!(cfg.listen_addr, "0.0.0.0:3000");
    unsafe {
        std::env::remove_var("LISTEN");
    }
}

#[test]
fn test_config_clone() {
    let cfg1 = Config::load();
    let cfg2 = cfg1.clone();
    assert_eq!(cfg1.listen_addr, cfg2.listen_addr);
}

#[test]
fn test_config_localhost_binding() {
    unsafe {
        std::env::set_var("LISTEN", "127.0.0.1:8000");
    }
    let cfg = Config::load();
    assert!(cfg.listen_addr.contains("127.0.0.1"));
    assert!(cfg.listen_addr.contains("8000"));
    unsafe {
        std::env::remove_var("LISTEN");
    }
}

#[test]
fn test_config_all_interfaces_binding() {
    unsafe {
        std::env::set_var("LISTEN", "0.0.0.0:5000");
    }
    let cfg = Config::load();
    assert!(cfg.listen_addr.starts_with("0.0.0.0"));
    unsafe {
        std::env::remove_var("LISTEN");
    }
}
