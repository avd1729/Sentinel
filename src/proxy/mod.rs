//! Reverse proxy functionality
//!
//! This module implements the core reverse proxy logic, including backend
//! management, load balancing, and request forwarding.

pub mod backend;
pub mod upstream;

pub use backend::{Backend, BackendPool, BackendState};
pub use upstream::ProxyHandler;
