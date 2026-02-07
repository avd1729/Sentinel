//! Backend server management
//!
//! This module manages the pool of backend servers, tracking their state
//! and selecting backends for incoming requests.

use crate::config::BackendConfig;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Represents the current state of a backend server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendState {
    /// Backend is healthy and accepting requests
    Up,
    /// Backend is down or unreachable
    Down,
}

/// Represents a backend server with its metadata
#[derive(Debug, Clone)]
pub struct Backend {
    /// Backend URL (e.g., "http://localhost:3000")
    pub url: String,
    
    /// Optional backend name for logging
    pub name: Option<String>,
    
    /// Current state of the backend
    pub state: BackendState,
    
    /// Last time backend was checked
    pub last_check: Option<Instant>,
    
    /// Number of consecutive failures
    pub consecutive_failures: u32,
}

impl Backend {
    /// Create a new backend from configuration
    pub fn new(config: BackendConfig) -> Self {
        Self {
            url: config.url,
            name: config.name,
            state: BackendState::Up,
            last_check: None,
            consecutive_failures: 0,
        }
    }

    /// Get a display name for the backend (name or URL)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.url)
    }

    /// Mark backend as failed
    pub fn mark_failed(&mut self) {
        self.consecutive_failures += 1;
        self.last_check = Some(Instant::now());
        
        // Mark as down after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.state = BackendState::Down;
            tracing::warn!(
                backend = self.display_name(),
                failures = self.consecutive_failures,
                "Backend marked as down"
            );
        }
    }

    /// Mark backend as successful
    pub fn mark_success(&mut self) {
        self.consecutive_failures = 0;
        self.last_check = Some(Instant::now());
        
        if self.state == BackendState::Down {
            self.state = BackendState::Up;
            tracing::info!(backend = self.display_name(), "Backend recovered");
        }
    }

    /// Check if backend is available for requests
    pub fn is_available(&self) -> bool {
        self.state == BackendState::Up
    }
}

/// Pool of backend servers
#[derive(Debug, Clone)]
pub struct BackendPool {
    backends: Arc<RwLock<Vec<Backend>>>,
    current_index: Arc<RwLock<usize>>,
}

impl BackendPool {
    /// Create a new backend pool from configuration
    pub fn new(configs: Vec<BackendConfig>) -> Self {
        let backends = configs.into_iter().map(Backend::new).collect();
        
        Self {
            backends: Arc::new(RwLock::new(backends)),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Select the next available backend using round-robin
    ///
    /// Returns None if no backends are available
    pub async fn select_backend(&self) -> Option<Backend> {
        let backends = self.backends.read().await;
        
        if backends.is_empty() {
            return None;
        }

        // Find first available backend starting from current index
        let mut index = *self.current_index.read().await;
        let start_index = index;
        
        loop {
            if backends[index].is_available() {
                let backend = backends[index].clone();
                
                // Update index for next request
                drop(backends);
                let mut current_index = self.current_index.write().await;
                *current_index = (index + 1) % self.backends.read().await.len();
                
                return Some(backend);
            }
            
            index = (index + 1) % backends.len();
            
            // If we've checked all backends and none are available
            if index == start_index {
                tracing::error!("No available backends in pool");
                return None;
            }
        }
    }

    /// Mark a backend as failed
    pub async fn mark_backend_failed(&self, backend_url: &str) {
        let mut backends = self.backends.write().await;
        
        if let Some(backend) = backends.iter_mut().find(|b| b.url == backend_url) {
            backend.mark_failed();
        }
    }

    /// Mark a backend as successful
    pub async fn mark_backend_success(&self, backend_url: &str) {
        let mut backends = self.backends.write().await;
        
        if let Some(backend) = backends.iter_mut().find(|b| b.url == backend_url) {
            backend.mark_success();
        }
    }

    /// Get all backends (for monitoring/debugging)
    pub async fn get_backends(&self) -> Vec<Backend> {
        self.backends.read().await.clone()
    }

    /// Get count of available backends
    pub async fn available_count(&self) -> usize {
        self.backends
            .read()
            .await
            .iter()
            .filter(|b| b.is_available())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_backend_failure_tracking() {
        let config = BackendConfig {
            url: "http://localhost:3000".to_string(),
            name: None,
        };
        
        let mut backend = Backend::new(config);
        
        // First failure
        backend.mark_failed();
        assert_eq!(backend.consecutive_failures, 1);
        assert!(backend.is_available());
        
        // Second failure
        backend.mark_failed();
        assert_eq!(backend.consecutive_failures, 2);
        assert!(backend.is_available());
        
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
        
        // Mark as failed
        backend.mark_failed();
        backend.mark_failed();
        backend.mark_failed();
        assert!(!backend.is_available());
        
        // Successful request recovers backend
        backend.mark_success();
        assert!(backend.is_available());
        assert_eq!(backend.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_backend_pool_selection() {
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
}
