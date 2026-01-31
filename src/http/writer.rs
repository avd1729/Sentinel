use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

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
    pub async fn write_to_stream(
        &mut self,
        stream: &mut TcpStream,
    ) -> anyhow::Result<()> {
        while self.written < self.buffer.len() {
            let write_fut = stream.write(&self.buffer[self.written..]);

            let n = match timeout(WRITE_TIMEOUT, write_fut).await {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => {
                    return Err(anyhow::anyhow!("write timed out"));
                }
            };

            if n == 0 {
                return Err(anyhow::anyhow!("connection closed while writing"));
            }

            self.written += n;
        }

        Ok(())
    }
}




