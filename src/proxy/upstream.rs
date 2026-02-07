//! Upstream connection and request forwarding
//!
//! This module handles connecting to backend servers and forwarding
//! HTTP requests/responses.

use crate::http::request::Request;
use crate::http::response::{Response, StatusCode};
use crate::proxy::backend::{Backend, BackendPool};
use anyhow::{Context, Result};
use bytes::{Buf, BytesMut};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Default buffer size for streaming
const BUFFER_SIZE: usize = 8192;

/// Handles proxying requests to backend servers
pub struct ProxyHandler {
    /// Pool of backend servers
    backend_pool: BackendPool,
    
    /// Connection timeout duration
    connection_timeout: Duration,
    
    /// Request timeout duration
    request_timeout: Duration,
}

impl ProxyHandler {
    /// Create a new proxy handler
    pub fn new(
        backend_pool: BackendPool,
        connection_timeout: Duration,
        request_timeout: Duration,
    ) -> Self {
        Self {
            backend_pool,
            connection_timeout,
            request_timeout,
        }
    }

    /// Forward an HTTP request to a backend server
    ///
    /// This function:
    /// 1. Selects an available backend
    /// 2. Connects to the backend
    /// 3. Forwards the request
    /// 4. Streams the response back
    /// 5. Retries with other backends if one fails
    pub async fn forward_request(&self, request: &Request) -> Result<Response> {
        let max_retries = self.backend_pool.available_count().await;
        
        if max_retries == 0 {
            anyhow::bail!("No available backends");
        }

        let mut last_error = None;
        
        // Try up to the number of available backends
        for attempt in 0..max_retries {
            // Select a backend
            let backend = match self.backend_pool.select_backend().await {
                Some(b) => b,
                None => {
                    tracing::error!("No available backends in pool");
                    break;
                }
            };

            tracing::debug!(
                backend = backend.display_name(),
                attempt = attempt + 1,
                max_retries = max_retries,
                method = ?request.method,
                path = %request.path,
                "Forwarding request to backend"
            );

            // Try to proxy the request
            match self.proxy_to_backend(&backend, request).await {
                Ok(response) => {
                    // Mark backend as successful
                    self.backend_pool.mark_backend_success(&backend.url).await;
                    
                    tracing::info!(
                        backend = backend.display_name(),
                        status = response.status.as_u16(),
                        method = ?request.method,
                        path = %request.path,
                        attempt = attempt + 1,
                        "Request forwarded successfully"
                    );
                    
                    return Ok(response);
                }
                Err(e) => {
                    // Mark backend as failed
                    self.backend_pool.mark_backend_failed(&backend.url).await;
                    
                    tracing::warn!(
                        backend = backend.display_name(),
                        error = %e,
                        method = ?request.method,
                        path = %request.path,
                        attempt = attempt + 1,
                        "Failed to proxy request to backend, will retry with another"
                    );
                    
                    last_error = Some(e);
                    // Continue to next backend
                }
            }
        }
        
        // All backends failed
        tracing::error!(
            method = ?request.method,
            path = %request.path,
            "All available backends failed"
        );
        
        // Return error from last attempt
        if let Some(e) = last_error {
            self.handle_proxy_error(&e)
        } else {
            self.handle_proxy_error(&anyhow::anyhow!("No available backends"))
        }
    }

    /// Proxy a request to a specific backend
    async fn proxy_to_backend(&self, backend: &Backend, request: &Request) -> Result<Response> {
        // Parse backend URL to get host and port
        let url = url::Url::parse(&backend.url)
            .context("Invalid backend URL")?;
        
        let host = url.host_str().context("Backend URL missing host")?;
        let port = url.port().unwrap_or(match url.scheme() {
            "https" => 443,
            _ => 80,
        });

        // Connect to backend with timeout
        let addr = format!("{}:{}", host, port);
        let stream = timeout(
            self.connection_timeout,
            TcpStream::connect(&addr),
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect to backend")?;

        tracing::trace!(backend = backend.display_name(), "Connected to backend");

        // Forward request and get response with timeout
        timeout(
            self.request_timeout,
            self.send_request_and_receive_response(stream, request, &url),
        )
        .await
        .context("Request timeout")?
    }

    /// Send request to backend and receive response
    async fn send_request_and_receive_response(
        &self,
        mut stream: TcpStream,
        request: &Request,
        backend_url: &url::Url,
    ) -> Result<Response> {
        // Build and send HTTP request
        let request_bytes = self.build_http_request(request, backend_url)?;
        stream.write_all(&request_bytes).await?;
        stream.flush().await?;

        tracing::trace!("Request sent to backend");

        // Read and parse response
        self.read_http_response(&mut stream).await
    }

    /// Build HTTP request bytes to send to backend
    /// 
    /// Note: This method is made public for integration testing purposes
    pub fn build_http_request(&self, request: &Request, backend_url: &url::Url) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Request line
        let method = format!("{:?}", request.method);
        let path = if request.path.is_empty() {
            "/"
        } else {
            &request.path
        };
        
        buffer.extend_from_slice(
            format!("{} {} {}\r\n", method, path, request.version).as_bytes()
        );

        // Headers - add/modify headers for backend
        let mut headers = request.headers.clone();
        
        // Set/update Host header to backend host
        if let Some(host) = backend_url.host_str() {
            let host_value = if let Some(port) = backend_url.port() {
                format!("{}:{}", host, port)
            } else {
                host.to_string()
            };
            headers.insert("Host".to_string(), host_value);
        }

        // Remove hop-by-hop headers
        headers.remove("Connection");
        headers.remove("Keep-Alive");
        headers.remove("Proxy-Connection");
        headers.remove("Transfer-Encoding");
        headers.remove("Upgrade");

        // Add Connection: close for simplicity
        headers.insert("Connection".to_string(), "close".to_string());

        // Write headers
        for (key, value) in &headers {
            buffer.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }

        // End of headers
        buffer.extend_from_slice(b"\r\n");

        // Body (if present)
        if !request.body.is_empty() {
            buffer.extend_from_slice(&request.body);
        }

        Ok(buffer)
    }

    /// Read HTTP response from backend
    async fn read_http_response(&self, stream: &mut TcpStream) -> Result<Response> {
        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        
        // Read response headers
        loop {
            let n = stream.read_buf(&mut buffer).await?;
            
            if n == 0 {
                anyhow::bail!("Connection closed before complete response received");
            }

            // Check if we've received complete headers (look for \r\n\r\n)
            if let Some(headers_end) = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                let headers_bytes = buffer.split_to(headers_end + 4);
                let (status, headers) = self.parse_response_headers(&headers_bytes)?;
                
                // Read body based on Content-Length
                let body = self.read_response_body(stream, &mut buffer, &headers).await?;
                
                // Build final response with body
                let response = Response::new(status)
                    .with_headers(headers)
                    .with_body(body)
                    .build();
                
                return Ok(response);
            }

            // Prevent unbounded header growth
            if buffer.len() > 64 * 1024 {
                anyhow::bail!("Response headers too large");
            }
        }
    }

    /// Parse response headers
    fn parse_response_headers(&self, headers_bytes: &[u8]) -> Result<(StatusCode, std::collections::HashMap<String, String>)> {
        let headers_str = std::str::from_utf8(headers_bytes)
            .context("Invalid UTF-8 in response headers")?;
        
        let mut lines = headers_str.lines();
        
        // Parse status line
        let status_line = lines.next().context("Empty response")?;
        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
        
        if parts.len() < 2 {
            anyhow::bail!("Invalid status line: {}", status_line);
        }

        let status_code: u16 = parts[1].parse()
            .context("Invalid status code")?;
        
        let status = match status_code {
            200 => StatusCode::Ok,
            201 => StatusCode::Created,
            204 => StatusCode::NoContent,
            400 => StatusCode::BadRequest,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            500..=599 => StatusCode::BadGateway, // Map all 5xx to BadGateway for now
            _ => StatusCode::Ok, // Default fallback
        };

        // Parse headers
        let mut headers = std::collections::HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(
                    key.trim().to_string(),
                    value.trim().to_string(),
                );
            }
        }

        Ok((status, headers))
    }

    /// Read response body based on Content-Length
    async fn read_response_body(
        &self,
        stream: &mut TcpStream,
        buffer: &mut BytesMut,
        headers: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<u8>> {
        // Check Content-Length header
        let content_length = if let Some(cl) = headers.get("Content-Length") {
            cl.parse::<usize>().unwrap_or(0)
        } else {
            // No Content-Length, read until connection closes
            let mut body = buffer.to_vec();
            loop {
                let n = stream.read_buf(buffer).await?;
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buffer[..n]);
                buffer.clear();
            }
            return Ok(body);
        };

        if content_length == 0 {
            return Ok(Vec::new());
        }

        let mut body = Vec::with_capacity(content_length);
        
        // Use existing buffer data first
        let from_buffer = buffer.len().min(content_length);
        body.extend_from_slice(&buffer[..from_buffer]);
        buffer.advance(from_buffer);

        // Read remaining bytes
        while body.len() < content_length {
            let remaining = content_length - body.len();
            let to_read = remaining.min(BUFFER_SIZE);
            
            buffer.resize(to_read, 0);
            let n = stream.read(&mut buffer[..to_read]).await?;
            
            if n == 0 {
                anyhow::bail!("Connection closed before complete body received");
            }
            
            body.extend_from_slice(&buffer[..n]);
        }

        Ok(body)
    }

    /// Handle proxy errors and return appropriate HTTP responses
    fn handle_proxy_error(&self, error: &anyhow::Error) -> Result<Response> {
        let error_str = error.to_string();
        
        // Determine appropriate status code based on error
        let (status, body) = if error_str.contains("timeout") {
            (
                StatusCode::GatewayTimeout,
                b"504 Gateway Timeout\r\n\r\nThe backend server did not respond in time.".to_vec(),
            )
        } else if error_str.contains("No available backends") || error_str.contains("All available backends failed") {
            (
                StatusCode::ServiceUnavailable,
                b"503 Service Unavailable\r\n\r\nNo backend servers are available.".to_vec(),
            )
        } else {
            (
                StatusCode::BadGateway,
                b"502 Bad Gateway\r\n\r\nFailed to connect to backend server.".to_vec(),
            )
        };

        Ok(Response::new(status)
            .with_header("Content-Type", "text/plain")
            .with_header("Content-Length", &body.len().to_string())
            .with_body(body)
            .build())
    }
}
