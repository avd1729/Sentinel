use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

use crate::http::parser::{parse_http_request, ParseError};
use crate::http::request::Request;

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::with_capacity(4096),
        }
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
}

