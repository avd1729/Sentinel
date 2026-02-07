//! Tests for backend pool management

use sentinel::config::BackendConfig;
use sentinel::proxy::backend::{Backend, BackendPool, BackendState};

#[test]
fn test_backend_creation() {
    let config = BackendConfig {
        url: "http://localhost:3000".to_string(),
        name: Some("backend-1".to_string()),
    };
    
    let backend = Backend::new(config);
    assert_eq!(backend.url, "http://localhost:3000");
    assert_eq!(backend.display_name(), "backend-1");
    assert!(backend.is_available());
}

#[test]
fn test_backend_creation_without_name() {
    let config = BackendConfig {
        url: "http://localhost:3001".to_string(),
        name: None,
    };
    
    let backend = Backend::new(config);
    assert_eq!(backend.url, "http://localhost:3001");
    assert_eq!(backend.display_name(), "http://localhost:3001");
    assert!(backend.is_available());
}

#[test]
fn test_backend_failure_tracking() {
    let config = BackendConfig {
        url: "http://localhost:3000".to_string(),
        name: None,
    };
    
    let mut backend = Backend::new(config);
    
    // Initial state
    assert_eq!(backend.consecutive_failures, 0);
    assert_eq!(backend.state, BackendState::Up);
    assert!(backend.is_available());
    
    // First failure
    backend.mark_failed();
    assert_eq!(backend.consecutive_failures, 1);
    assert!(backend.is_available());
    assert_eq!(backend.state, BackendState::Up);
    
    // Second failure
    backend.mark_failed();
    assert_eq!(backend.consecutive_failures, 2);
    assert!(backend.is_available());
    assert_eq!(backend.state, BackendState::Up);
    
    // Third failure - should mark as down
    backend.mark_failed();
    assert_eq!(backend.consecutive_failures, 3);
    assert!(!backend.is_available());
    assert_eq!(backend.state, BackendState::Down);
}

#[test]
fn test_backend_recovery() {
    let config = BackendConfig {
        url: "http://localhost:3000".to_string(),
        name: None,
    };
    
    let mut backend = Backend::new(config);
    
    // Mark as failed multiple times
    backend.mark_failed();
    backend.mark_failed();
    backend.mark_failed();
    assert!(!backend.is_available());
    assert_eq!(backend.state, BackendState::Down);
    
    // Successful request recovers backend
    backend.mark_success();
    assert!(backend.is_available());
    assert_eq!(backend.consecutive_failures, 0);
    assert_eq!(backend.state, BackendState::Up);
}

#[test]
fn test_backend_partial_failure_recovery() {
    let config = BackendConfig {
        url: "http://localhost:3000".to_string(),
        name: None,
    };
    
    let mut backend = Backend::new(config);
    
    // Fail once
    backend.mark_failed();
    assert_eq!(backend.consecutive_failures, 1);
    assert!(backend.is_available());
    
    // Recover
    backend.mark_success();
    assert_eq!(backend.consecutive_failures, 0);
    assert!(backend.is_available());
}

#[tokio::test]
async fn test_backend_pool_creation() {
    let configs = vec![
        BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: Some("backend-1".to_string()),
        },
        BackendConfig {
            url: "http://localhost:3001".to_string(),
            name: Some("backend-2".to_string()),
        },
    ];
    
    let pool = BackendPool::new(configs);
    let count = pool.available_count().await;
    
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_backend_pool_round_robin_selection() {
    let configs = vec![
        BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: Some("backend-1".to_string()),
        },
        BackendConfig {
            url: "http://localhost:3001".to_string(),
            name: Some("backend-2".to_string()),
        },
    ];
    
    let pool = BackendPool::new(configs);
    
    // Should select backends in round-robin
    let backend1 = pool.select_backend().await.unwrap();
    let backend2 = pool.select_backend().await.unwrap();
    let backend3 = pool.select_backend().await.unwrap();
    
    assert_eq!(backend1.url, "http://localhost:3000");
    assert_eq!(backend2.url, "http://localhost:3001");
    assert_eq!(backend3.url, "http://localhost:3000"); // Wraps around
}

#[tokio::test]
async fn test_backend_pool_skips_unavailable() {
    let configs = vec![
        BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: Some("backend-1".to_string()),
        },
        BackendConfig {
            url: "http://localhost:3001".to_string(),
            name: Some("backend-2".to_string()),
        },
        BackendConfig {
            url: "http://localhost:3002".to_string(),
            name: Some("backend-3".to_string()),
        },
    ];
    
    let pool = BackendPool::new(configs);
    
    // Mark backend-2 as failed
    pool.mark_backend_failed("http://localhost:3001").await;
    pool.mark_backend_failed("http://localhost:3001").await;
    pool.mark_backend_failed("http://localhost:3001").await;
    
    // Should skip failed backend
    let backend1 = pool.select_backend().await.unwrap();
    let backend2 = pool.select_backend().await.unwrap();
    let backend3 = pool.select_backend().await.unwrap();
    
    assert_eq!(backend1.url, "http://localhost:3000");
    assert_eq!(backend2.url, "http://localhost:3002");
    assert_eq!(backend3.url, "http://localhost:3000");
}

#[tokio::test]
async fn test_backend_pool_no_available_backends() {
    let configs = vec![
        BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: Some("backend-1".to_string()),
        },
    ];
    
    let pool = BackendPool::new(configs);
    
    // Mark the only backend as failed
    pool.mark_backend_failed("http://localhost:3000").await;
    pool.mark_backend_failed("http://localhost:3000").await;
    pool.mark_backend_failed("http://localhost:3000").await;
    
    // Should return None
    let backend = pool.select_backend().await;
    assert!(backend.is_none());
    
    let count = pool.available_count().await;
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_backend_pool_mark_success() {
    let configs = vec![
        BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: Some("backend-1".to_string()),
        },
    ];
    
    let pool = BackendPool::new(configs);
    
    // Mark as failed
    pool.mark_backend_failed("http://localhost:3000").await;
    pool.mark_backend_failed("http://localhost:3000").await;
    pool.mark_backend_failed("http://localhost:3000").await;
    
    assert_eq!(pool.available_count().await, 0);
    
    // Mark as successful - should recover
    pool.mark_backend_success("http://localhost:3000").await;
    
    assert_eq!(pool.available_count().await, 1);
}
