use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::http::response::{Response, StatusCode};

const HTTP_VERSION: &str = "HTTP/1.1";

fn serialize_response(resp: &Response) -> Vec<u8> {
    let mut buf = Vec::new();

    // Status line
    let status_line = format!(
        "{} {} {}\r\n",
        HTTP_VERSION,
        resp.status.as_u16(),
        resp.status.reason_phrase()
    );
    buf.extend_from_slice(status_line.as_bytes());

    // Headers
    for (k, v) in &resp.headers {
        buf.extend_from_slice(k.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(v.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    // Header/body separator
    buf.extend_from_slice(b"\r\n");

    // Body
    buf.extend_from_slice(&resp.body);

    buf
}

pub struct ResponseWriter {
    buffer: Vec<u8>,
    written: usize,
}

impl ResponseWriter {
    pub fn new(response: &Response) -> Self {
        Self {
            buffer: serialize_response(response),
            written: 0,
        }
    }

    pub async fn write_to_stream(
        &mut self,
        stream: &mut TcpStream,
    ) -> anyhow::Result<()> {
        while self.written < self.buffer.len() {
            let n = stream
                .write(&self.buffer[self.written..])
                .await?;

            if n == 0 {
                return Err(anyhow::anyhow!("connection closed while writing"));
            }

            self.written += n;
        }

        Ok(())
    }
}



