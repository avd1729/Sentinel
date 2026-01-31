//! HTTP protocol implementation.
//!
//! This module implements a complete HTTP/1.1 server with support for keep-alive connections.
//!
//! # Architecture
//!
//! The HTTP layer is organized into several submodules:
//!
//! - **`connection`**: The main connection handler implementing the request-response state machine
//! - **`parser`**: Parses incoming HTTP requests from byte buffers
//! - **`request`**: HTTP request representation and parsing utilities
//! - **`response`**: HTTP response representation with builder pattern
//! - **`writer`**: Serializes and writes HTTP responses to the client
//! - **`mime`**: MIME type detection based on file extensions
//!
//! # Connection State Machine
//!
//! Each client connection goes through a state machine:
//!
//! ```text
//!        ┌─────────────┐
//!        │   Reading   │ ← Wait for incoming request data
//!        └──────┬──────┘
//!               │ Request received
//!               ▼
//!        ┌──────────────────┐
//!        │   Processing     │ ← Generate response
//!        └──────┬───────────┘
//!               │ Response ready
//!               ▼
//!        ┌──────────────────┐
//!        │    Writing       │ ← Send response to client
//!        └──────┬───────────┘
//!               │ Response sent
//!               ├─ Keep-Alive → Reading (same connection)
//!               └─ Close → Closed
//! ```
//!
//! # Example
//!
//! ```ignore
//! use sentinel::http::connection::Connection;
//! use tokio::net::TcpListener;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!     loop {
//!         let (socket, _addr) = listener.accept().await?;
//!         tokio::spawn(async move {
//!             let mut conn = Connection::new(socket);
//!             if let Err(e) = conn.run().await {
//!                 eprintln!("Connection error: {}", e);
//!             }
//!         });
//!     }
//! }
//! ```

pub mod request;
pub mod response;
pub mod parser;
pub mod connection;
pub mod writer;
pub mod mime;
