use std::collections::HashMap;

/// HTTP status codes supported by the server.
///
/// Common HTTP status codes used in responses:
/// - `Ok` (200): Request successful
/// - `Created` (201): Resource created successfully
/// - `NoContent` (204): Successful request with no content
/// - `BadRequest` (400): Malformed request
/// - `NotFound` (404): Resource not found
/// - `MethodNotAllowed` (405): HTTP method not supported
/// - `InternalServerError` (500): Server error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    /// 200 OK
    Ok,
    /// 201 Created
    Created,
    /// 204 No Content
    NoContent,
    /// 400 Bad Request
    BadRequest,
    /// 404 Not Found
    NotFound,
    /// 405 Method Not Allowed
    MethodNotAllowed,
    /// 500 Internal Server Error
    InternalServerError
}

impl StatusCode {
    /// Returns the numeric HTTP status code.
    ///
    /// # Example
    ///
    /// ```
    /// # use sentinel::http::response::StatusCode;
    /// assert_eq!(StatusCode::Ok.as_u16(), 200);
    /// assert_eq!(StatusCode::NotFound.as_u16(), 404);
    /// ```
    pub fn as_u16(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::NoContent => 204,
            StatusCode::BadRequest => 400,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::InternalServerError => 500,
        }
    }

    /// Returns the standard HTTP reason phrase for this status code.
    ///
    /// # Example
    ///
    /// ```
    /// # use sentinel::http::response::StatusCode;
    /// assert_eq!(StatusCode::Ok.reason_phrase(), "OK");
    /// assert_eq!(StatusCode::NotFound.reason_phrase(), "Not Found");
    /// ```
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::NoContent => "No Content",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::InternalServerError => "Internal Server Error",
        }
    }
}

/// Represents a complete HTTP response ready to be sent to a client.
///
/// Contains the HTTP status code, headers, and response body.
#[derive(Debug)]
pub struct Response {
    /// The HTTP status code
    pub status: StatusCode,
    /// HTTP headers as key-value pairs
    pub headers: HashMap<String, String>,
    /// Response body as bytes
    pub body: Vec<u8>,
}

/// Builder for constructing HTTP responses in a fluent style.
///
/// # Example
///
/// ```ignore
/// let response = ResponseBuilder::new(StatusCode::Ok)
///     .header("Content-Type", "application/json")
///     .body(b"{}".to_vec())
///     .build();
/// ```
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl ResponseBuilder {
    /// Creates a new response builder with the specified status code.
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Adds or replaces a header.
    ///
    /// # Arguments
    ///
    /// * `key` - Header name (case-insensitive in HTTP)
    /// * `value` - Header value
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.header("Content-Type", "text/plain")
    ///     .header("Cache-Control", "no-cache")
    /// ```
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the response body.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Builds the final Response.
    ///
    /// Automatically adds the Content-Length header based on body size if not already present.
    pub fn build(mut self) -> Response {
        // Auto Content-Length (important)
        self.headers
            .entry("Content-Length".to_string())
            .or_insert_with(|| self.body.len().to_string());

        Response {
            status: self.status,
            headers: self.headers,
            body: self.body,
        }
    }
}

impl Response {
    /// Creates a simple 200 OK response with the given body.
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        ResponseBuilder::new(StatusCode::Ok)
            .body(body.into())
            .build()
    }

    /// Creates a 404 Not Found response.
    pub fn not_found() -> Self {
        ResponseBuilder::new(StatusCode::NotFound)
            .body(b"404 Not Found".to_vec())
            .build()
    }

    /// Creates a 500 Internal Server Error response.
    pub fn internal_error() -> Self {
        ResponseBuilder::new(StatusCode::InternalServerError)
            .body(b"500 Internal Server Error".to_vec())
            .build()
    }
}


