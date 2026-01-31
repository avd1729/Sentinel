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


pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    state: ConnectionState,
}

pub enum ConnectionState {
    Reading,
    Processing(Request),
    Writing(Response, bool), // Response and keep_alive?
    Closed,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::with_capacity(4096),
            state: ConnectionState::Reading,
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            match std::mem::replace(&mut self.state, ConnectionState::Reading) {
                ConnectionState::Reading => {
                    tracing::debug!("Connection state: Reading");
                    match self.read_request().await? {
                        Some(req) => {
                            tracing::info!("Received request: {:?} {}", req.method, req.path);
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
                    let (response, keep_alive) = Self::handle_request(&req).await;
                    tracing::info!("Sending response: {}", response.status.as_u16());
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

    async fn handle_request(req: &Request) -> (Response, bool) {
        let keep_alive = req.keep_alive();

        // Only GET supported
        if req.method != Method::GET {
            return (
                ResponseBuilder::new(StatusCode::MethodNotAllowed)
                    .body(b"405 Method Not Allowed".to_vec())
                    .build(),
                keep_alive,
            );
        }

        // Normalize path
        let mut path = req.path.clone();
        if path == "/" {
            path = "/index.html".to_string();
        }

        // Prevent path traversal
        if path.contains("..") {
            return (
                ResponseBuilder::new(StatusCode::BadRequest)
                    .body(b"400 Bad Request".to_vec())
                    .build(),
                keep_alive,
            );
        }

        let full_path: PathBuf = Path::new("public").join(&path[1..]);

        match fs::read(&full_path).await {
            Ok(contents) => {
                let mime = content_type(&path);
                let response = ResponseBuilder::new(StatusCode::Ok)
                    .header("Content-Type", mime)
                    .body(contents)
                    .build();

                (response, keep_alive)
            }

            Err(_) => (
                ResponseBuilder::new(StatusCode::NotFound)
                    .body(b"404 Not Found".to_vec())
                    .build(),
                keep_alive,
            ),
        }
    }
}

