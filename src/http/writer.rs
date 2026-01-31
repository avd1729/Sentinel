use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

use crate::http::response::{Response, StatusCode};

const HTTP_VERSION: &str = "HTTP/1.1";

/// Serializes an HTTP response into a byte buffer.
///
/// Converts a Response struct into HTTP wire format including:
/// - Status line (HTTP/1.1 status_code reason_phrase)
/// - Headers (key: value)
/// - Auto-added Content-Length if not present
/// - Auto-added Connection: close if not present
/// - Blank line separator
/// - Response body
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

    let mut has_len = false;
    let mut has_conn = false;

    for (k, v) in &resp.headers {
        if k.eq_ignore_ascii_case("content-length") {
            has_len = true;
        }
        if k.eq_ignore_ascii_case("connection") {
            has_conn = true;
        }

        buf.extend_from_slice(k.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(v.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    // REQUIRED headers
    if !has_len {
        buf.extend_from_slice(
            format!("Content-Length: {}\r\n", resp.body.len()).as_bytes()
        );
    }

    if !has_conn {
        buf.extend_from_slice(b"Connection: close\r\n");
    }

    // End headers
    buf.extend_from_slice(b"\r\n");

    // Body
    buf.extend_from_slice(&resp.body);

    buf
}

/// Handles writing HTTP responses to a TCP stream.
///
/// This struct manages the serialization and transmission of an HTTP response
/// to a client. It handles partial writes by tracking how many bytes have been
/// sent and applying a 5-second timeout to write operations.
///
/// # Example
///
/// ```ignore
/// use sentinel::http::writer::ResponseWriter;
/// use sentinel::http::response::{Response, ResponseBuilder, StatusCode};
///
/// let response = ResponseBuilder::new(StatusCode::Ok)
///     .body(b"Hello, World!".to_vec())
///     .build();
///
/// let mut writer = ResponseWriter::new(&response);
/// writer.write_to_stream(&mut stream).await?;
/// ```
pub struct ResponseWriter {
    buffer: Vec<u8>,
    written: usize,
}

impl ResponseWriter {

    /// Creates a new ResponseWriter from a Response.
    ///
    /// Serializes the response into HTTP wire format and prepares it for transmission.
    pub fn new(response: &Response) -> Self {
        Self {
            buffer: serialize_response(response),
            written: 0,
        }
    }

    /// Writes the complete response to the TCP stream.
    ///
    /// Handles partial writes by tracking progress. If the underlying socket
    /// cannot accept all data at once, this function will resume writing on
    /// subsequent calls. Each write operation has a 5-second timeout.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream to write to
    ///
    /// # Returns
    ///
    /// `Ok(())` when all bytes have been successfully written, or an error
    /// if I/O fails or the write times out.
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




