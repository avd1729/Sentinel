use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

use crate::http::parser::{parse_http_request, ParseError};
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::writer::ResponseWriter;

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    state: ConnectionState,
}

pub enum ConnectionState {
    Reading,
    Processing(Request),
    Writing(ResponseWriter),
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
            match &mut self.state {
                ConnectionState::Reading => {
                    match self.read_request().await? {
                        Some(req) => {
                            self.state = ConnectionState::Processing(req);
                        }
                        None => {
                            self.state = ConnectionState::Closed;
                        }
                    }
                }

                ConnectionState::Processing(req) => {
                    // TEMP handler (real routing comes later)
                    let response = Self::handle_request(req);

                    let writer = ResponseWriter::new(&response);
                    self.state = ConnectionState::Writing(writer);
                }

                ConnectionState::Writing(writer) => {
                    writer.write_to_stream(&mut self.stream).await?;
                    self.state = ConnectionState::Closed;
                }

                ConnectionState::Closed => {
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

    fn handle_request(_req: &Request) -> Response {
        Response::ok("Hello from Sentinel\n")
    }
}

