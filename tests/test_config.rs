use sentinel::config::Config;
use std::fs;
use std::sync::Mutex;

// Global lock to ensure tests don't interfere with each other
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_no_config_file<F>(test: F) 
where
    F: FnOnce(),
{
    let _lock = TEST_LOCK.lock().unwrap();
    
    // Backup config.yaml if it exists
    let config_exists = fs::metadata("config.yaml").is_ok();
    if config_exists {
        fs::rename("config.yaml", "config.yaml.test_backup").expect("Failed to backup config.yaml");
    }
    
    // Run the test
    test();
    
    // Restore config.yaml
    if config_exists {
        fs::rename("config.yaml.test_backup", "config.yaml").expect("Failed to restore config.yaml");
    }
}

#[test]
fn test_config_default_address() {
    with_no_config_file(|| {
        unsafe {
            std::env::remove_var("LISTEN");
        }
        let cfg = Config::load();
        assert_eq!(cfg.server.listen_addr, "127.0.0.1:8080");
        assert_eq!(cfg.static_files.root.to_str().unwrap(), "public");
        assert_eq!(cfg.static_files.index, "index.html");
    });
}

#[test]
fn test_config_custom_address_from_env() {
    with_no_config_file(|| {
        unsafe {
            std::env::set_var("LISTEN", "0.0.0.0:3000");
        }
        let cfg = Config::load();
        assert_eq!(cfg.server.listen_addr, "0.0.0.0:3000");
        unsafe {
            std::env::remove_var("LISTEN");
        }
    });
}

#[test]
fn test_config_clone() {
    let cfg1 = Config::load();
    let cfg2 = cfg1.clone();
    assert_eq!(cfg1.server.listen_addr, cfg2.server.listen_addr);
    assert_eq!(cfg1.static_files.index, cfg2.static_files.index);
}

#[test]
fn test_config_localhost_binding() {
    with_no_config_file(|| {
        unsafe {
            std::env::set_var("LISTEN", "127.0.0.1:8000");
        }
        let cfg = Config::load();
        assert!(cfg.server.listen_addr.contains("127.0.0.1"));
        assert!(cfg.server.listen_addr.contains("8000"));
        unsafe {
            std::env::remove_var("LISTEN");
        }
    });
}

#[test]
fn test_config_all_interfaces_binding() {
    with_no_config_file(|| {
        unsafe {
            std::env::set_var("LISTEN", "0.0.0.0:5000");
        }
        let cfg = Config::load();
        assert!(cfg.server.listen_addr.starts_with("0.0.0.0"));
        unsafe {
            std::env::remove_var("LISTEN");
        }
    });
}

#[test]
fn test_config_from_yaml() {
    let _lock = TEST_LOCK.lock().unwrap();
    use std::path::PathBuf;
    
    let yaml_content = r#"
server:
  listen_addr: "0.0.0.0:9000"

static_files:
  root: "www"
  index: "home.html"
  error_pages:
    not_found: "errors/404.html"
    bad_request: "errors/400.html"
  directory_listing: false
"#;
    
    fs::write("test_config.yaml", yaml_content).unwrap();
    let cfg = Config::load_from_file("test_config.yaml").unwrap();
    
    assert_eq!(cfg.server.listen_addr, "0.0.0.0:9000");
    assert_eq!(cfg.static_files.root, PathBuf::from("www"));
    assert_eq!(cfg.static_files.index, "home.html");
    assert_eq!(cfg.static_files.error_pages.not_found, Some("errors/404.html".to_string()));
    assert_eq!(cfg.static_files.error_pages.bad_request, Some("errors/400.html".to_string()));
    assert_eq!(cfg.static_files.directory_listing, false);
    
    fs::remove_file("test_config.yaml").unwrap();
}

#[test]
fn test_config_yaml_priority() {
    let _lock = TEST_LOCK.lock().unwrap();
    
    let yaml_content = r#"
server:
  listen_addr: "192.168.1.1:7777"

static_files:
  root: "test_public"
  index: "test.html"
  directory_listing: false
"#;
    
    fs::write("test_priority.yaml", yaml_content).unwrap();
    
    unsafe {
        std::env::set_var("LISTEN", "0.0.0.0:9999");
    }
    
    let cfg = Config::load_from_file("test_priority.yaml").unwrap();
    assert_eq!(cfg.server.listen_addr, "192.168.1.1:7777");
    assert_eq!(cfg.static_files.index, "test.html");
    
    unsafe {
        std::env::remove_var("LISTEN");
    }
    fs::remove_file("test_priority.yaml").unwrap();
}

