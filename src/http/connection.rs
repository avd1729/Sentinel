use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

use crate::http::parser::{parse_http_request, ParseError};
use crate::http::request::Request;
use crate::http::writer::ResponseWriter;

use std::path::{Path, PathBuf};
use tokio::fs;

use crate::http::mime::content_type;
use crate::http::request::Method;
use crate::http::response::{Response, ResponseBuilder, StatusCode};
use crate::config::StaticFilesConfig;
use std::time::Instant;

/// Handles a single HTTP client connection with support for keep-alive and pipelining.
///
/// The `Connection` manages the lifecycle of a TCP connection, reading HTTP requests,
/// processing them, and sending responses back to the client. It implements a state
/// machine to handle the various stages of request/response processing.
///
/// # State Machine
///
/// The connection cycles through these states:
///
/// 1. **Reading**: Reads data from the client and parses incoming HTTP requests
/// 2. **Processing**: Handles the parsed request and generates a response
/// 3. **Writing**: Sends the HTTP response back to the client
/// 4. **Closed**: Connection is being terminated
///
/// The machine allows keep-alive connections to cycle back from Writing to Reading
/// for multiple requests on the same connection.
///
/// # Example
///
/// ```ignore
/// use sentinel::http::connection::Connection;
/// use tokio::net::TcpListener;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let listener = TcpListener::bind("127.0.0.1:8080").await?;
///     
///     loop {
///         let (socket, _) = listener.accept().await?;
///         tokio::spawn(async move {
///             let mut conn = Connection::new(socket);
///             let _ = conn.run().await;
///         });
///     }
/// }
/// ```
pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    state: ConnectionState,
    request_start: Option<Instant>,
    static_config: StaticFilesConfig,
}

/// Represents the state of an HTTP connection in its processing lifecycle.
///
/// - `Reading`: Awaiting HTTP request data from the client
/// - `Processing`: Handling a received request and preparing a response
/// - `Writing`: Sending the response back to the client
/// - `Closed`: Connection should be terminated
#[derive(Debug)]
pub enum ConnectionState {
    /// Reading state: Connection is waiting for HTTP request data
    Reading,
    /// Processing state: A complete request has been parsed and needs handling
    Processing(Request),
    /// Writing state: A response is ready to be sent (response, keep_alive flag)
    Writing(Response, bool),
    /// Closed state: Connection should be closed
    Closed,
}

impl Connection {
    /// Creates a new HTTP connection handler for the given TCP stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream connected to the client
    /// * `static_config` - Configuration for serving static files
    ///
    /// # Returns
    ///
    /// A new `Connection` initialized with the provided stream and ready to handle requests.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (socket, _) = listener.accept().await?;
    /// let mut conn = Connection::new(socket, static_config);
    /// conn.run().await?;
    /// ```
    pub fn new(stream: TcpStream, static_config: StaticFilesConfig) -> Self {
        Self {
            stream,
            buffer: Vec::with_capacity(4096),
            state: ConnectionState::Reading,
            request_start: None,
            static_config,
        }
    }

    /// Runs the connection state machine until the connection closes.
    ///
    /// This function implements the HTTP protocol handling loop, cycling through states:
    /// 1. Reading - waits for and parses HTTP requests
    /// 2. Processing - handles the request and generates responses
    /// 3. Writing - sends the response to the client
    /// 4. Closed - terminates
    ///
    /// The function supports HTTP keep-alive connections, allowing multiple requests
    /// on the same connection. Each request is logged with method, path, status code,
    /// and duration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the connection is normally closed, or an error if
    /// an I/O error or protocol violation occurs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - TCP read/write operations fail
    /// - Invalid HTTP is received
    /// - Backend file operations fail
    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            match std::mem::replace(&mut self.state, ConnectionState::Reading) {
                ConnectionState::Reading => {
                    tracing::debug!("Connection state: Reading");
                    match self.read_request().await? {
                        Some(req) => {
                            self.request_start = Some(Instant::now());
                            tracing::info!(
                                method = ?req.method,
                                path = %req.path,
                                "Received HTTP request"
                            );
                            self.state = ConnectionState::Processing(req);
                        }
                        None => {
                            tracing::debug!("Client closed connection");
                            self.state = ConnectionState::Closed;
                        }
                    }
                }

                ConnectionState::Processing(req) => {
                    tracing::debug!("Connection state: Processing");
                    // TEMP handler (real routing comes later)
                    let (response, keep_alive) = self.handle_request(&req).await;
                    let status = response.status.as_u16();
                    
                    if let Some(start) = self.request_start.take() {
                        let duration = start.elapsed();
                        tracing::info!(
                            method = ?req.method,
                            path = %req.path,
                            status = status,
                            duration_ms = duration.as_millis(),
                            "HTTP request completed"
                        );
                    }
                    
                    self.state = ConnectionState::Writing(response, keep_alive);
                }

                ConnectionState::Writing(response, keep_alive) => {
                    tracing::debug!("Connection state: Writing");
                    let mut writer = ResponseWriter::new(&response);
                    writer.write_to_stream(&mut self.stream).await?;
                    tracing::debug!("Response written, keep_alive: {}", keep_alive);

                    if keep_alive {
                        self.state = ConnectionState::Reading; // go back for next request
                    } else {
                        self.state = ConnectionState::Closed;
                    }
                }

                ConnectionState::Closed => {
                    tracing::debug!("Connection state: Closed");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Reads and parses a complete HTTP request from the client.
    ///
    /// This function implements non-blocking request reading with buffering. It continues
    /// to read from the socket until a complete HTTP request is received and successfully parsed.
    ///
    /// The function uses an internal buffer to handle partial reads and uses the HTTP parser
    /// to detect when a complete request has been received.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(request))` - A complete, valid HTTP request has been parsed
    /// - `Ok(None)` - The client closed the connection before sending a request
    /// - `Err(e)` - An I/O error occurred or the HTTP is malformed
    ///
    /// # Example
    ///
    /// ```ignore
    /// match conn.read_request().await? {
    ///     Some(req) => println!("Got request: {:?} {}", req.method, req.path),
    ///     None => println!("Client disconnected"),
    /// }
    /// ```
    pub async fn read_request(&mut self) -> anyhow::Result<Option<Request>> {
        loop {
            // Try parsing whatever we already have
            match parse_http_request(&self.buffer) {
                Ok((request, consumed)) => {
                    // Remove consumed bytes
                    self.buffer.drain(..consumed);
                    return Ok(Some(request));
                }

                Err(ParseError::Incomplete) => {
                    // Need more data → fall through to read
                }

                Err(e) => {
                    // Malformed request → protocol error
                    return Err(anyhow::anyhow!("HTTP parse error: {:?}", e));
                }
            }

            // Read more data
            let mut temp = [0u8; 1024];
            let n = self.stream.read(&mut temp).await?;

            if n == 0 {
                // Client closed connection
                return Ok(None);
            }

            self.buffer.extend_from_slice(&temp[..n]);
        }
    }

    /// Handles an HTTP request and generates an appropriate response.
    ///
    /// Currently serves files from the `./public` directory as a temporary implementation.
    /// In Phase 2, this will be replaced with backend proxying logic to forward requests
    /// to configured backend servers based on the routing configuration.
    ///
    /// Supports all HTTP methods (GET, POST, PUT, DELETE, etc.) and will forward them
    /// appropriately once backend connection handling is implemented.
    ///
    /// # Arguments
    ///
    /// * `req` - The parsed HTTP request to handle
    ///
    /// # Returns
    ///
    /// A tuple of `(Response, bool)` where:
    /// - `Response` is the generated HTTP response
    /// - `bool` indicates whether keep-alive is enabled for this connection
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (response, keep_alive) = Connection::handle_request(&request).await;
    /// if keep_alive {
    ///     // Can reuse connection for next request
    /// }
    /// ```
    async fn handle_request(&self, req: &Request) -> (Response, bool) {
        let keep_alive = req.keep_alive();

        // TODO: Phase 2 - Replace with backend proxying logic
        // For now, serve static files based on configuration
        
        // Normalize path
        let mut path = req.path.clone();
        if path == "/" {
            path = format!("/{}", self.static_config.index);
        }

        // Prevent path traversal
        if path.contains("..") {
            let error_body = if let Some(ref error_page) = self.static_config.error_pages.bad_request {
                let error_path = self.static_config.root.join(error_page);
                fs::read(&error_path).await.unwrap_or_else(|_| b"400 Bad Request".to_vec())
            } else {
                b"400 Bad Request".to_vec()
            };
            
            return (
                ResponseBuilder::new(StatusCode::BadRequest)
                    .body(error_body)
                    .build(),
                keep_alive,
            );
        }

        let full_path: PathBuf = self.static_config.root.join(&path[1..]);

        match fs::read(&full_path).await {
            Ok(contents) => {
                let mime = content_type(&path);
                let response = ResponseBuilder::new(StatusCode::Ok)
                    .header("Content-Type", mime)
                    .body(contents)
                    .build();

                (response, keep_alive)
            }

            Err(_) => {
                let error_body = if let Some(ref error_page) = self.static_config.error_pages.not_found {
                    let error_path = self.static_config.root.join(error_page);
                    fs::read(&error_path).await.unwrap_or_else(|_| b"404 Not Found".to_vec())
                } else {
                    b"404 Not Found".to_vec()
                };
                
                (
                    ResponseBuilder::new(StatusCode::NotFound)
                        .body(error_body)
                        .build(),
                    keep_alive,
                )
            }
        }
    }
}

