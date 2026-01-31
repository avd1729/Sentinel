use crate::http::request::{Method, Request};
use std::collections::HashMap;

#[derive(Debug)]
pub enum ParseError {
    InvalidRequest,
    InvalidMethod,
    InvalidHeader,
    InvalidContentLength,
    Incomplete,
}

pub fn parse_http_request(buf: &[u8]) -> Result<(Request, usize), ParseError> {

    // Look for header/body separator
    let headers_end = find_headers_end(buf).ok_or(ParseError::Incomplete)?;
    let header_bytes = &buf[..headers_end];
    let body_bytes = &buf[headers_end + 4..];

    let headers_str = std::str::from_utf8(header_bytes)
        .map_err(|_| ParseError::InvalidRequest)?;

    let mut lines = headers_str.split("\r\n");

    // Request line
    let request_line = lines.next().ok_or(ParseError::InvalidRequest);
    let mut parts = request_line?.split_whitespace();

    let method_str = parts.next().ok_or(ParseError::InvalidRequest)?;
    let path = parts.next().ok_or(ParseError::InvalidRequest)?;
    let version = parts.next().ok_or(ParseError::InvalidRequest)?;

    let method = Method::from_str(method_str).ok_or(ParseError::InvalidMethod)?;

    // Headers
    let mut headers = HashMap::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        let (key, value) = line
            .split_once(':')
            .ok_or(ParseError::InvalidHeader)?;

        headers.insert(
           key.trim().to_string(),
           value.trim().to_string(),
        );
    }

    // Body
    let content_length = headers
        .get("Content-Length")
        .map(|v| v.parse::<usize>().map_err(|_| ParseError::InvalidContentLength))
        .transpose()?
        .unwrap_or(0);

    if body_bytes.len() < content_length {
        return Err(ParseError::Incomplete);
    }

    let body = body_bytes[..content_length].to_vec();

    let request = Request {
        method,
        path: path.to_string(),
        version: version.to_string(),
        headers,
        body,
    };

    let total_consumed = headers_end + 4 + content_length;
    Ok((request, total_consumed))

}

fn find_headers_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_get() {
        let req = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";

        let (parsed, consumed) = parse_http_request(req).unwrap();

        assert_eq!(parsed.path, "/");
        assert_eq!(parsed.headers.get("Host").unwrap(), "example.com");
        assert_eq!(consumed, req.len());
    }
}

