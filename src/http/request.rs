use std::collections::HashMap;

/// HTTP request methods.
///
/// Represents the HTTP method/verb of a request. Currently the server fully
/// supports GET. Other methods are parsed but may return 405 Method Not Allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    /// GET - Retrieve a resource
    GET,
    /// POST - Create or submit data
    POST,
    /// PUT - Replace a resource
    PUT,
    /// DELETE - Delete a resource
    DELETE,
    /// HEAD - Like GET but without the response body
    HEAD,
    /// OPTIONS - Describe communication options
    OPTIONS,
    /// PATCH - Partial modification of a resource
    PATCH
}

/// Represents a parsed HTTP request from a client.
///
/// Contains all information extracted from the HTTP request line and headers.
/// The body field contains any request entity (e.g., for POST/PUT requests).
#[derive(Debug, Clone)]
pub struct Request {
    /// The HTTP method (GET, POST, etc.)
    pub method: Method,
    /// The request path/URL (e.g., "/index.html")
    pub path: String,
    /// HTTP version (typically "HTTP/1.1")
    pub version: String,
    /// Request headers as key-value pairs
    pub headers: HashMap<String, String>,
    /// Request body for POST/PUT requests
    pub body: Vec<u8>,
}

/// Builder for constructing Request objects.
pub struct RequestBuilder {
    method: Option<Method>,
    path: Option<String>,
    version: Option<String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Method {
    /// Parses an HTTP method from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - String representation of the method (case-sensitive, typically uppercase)
    ///
    /// # Returns
    ///
    /// `Some(Method)` if the string matches a known method, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// # use sentinel::http::request::Method;
    /// assert_eq!(Method::from_str("GET"), Some(Method::GET));
    /// assert_eq!(Method::from_str("get"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "HEAD" => Some(Method::HEAD),
            "OPTIONS" => Some(Method::OPTIONS),
            "PATCH" => Some(Method::PATCH),
            _ => None,
        }
    }
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            method: None,
            path: None,
            version: None,
            headers: HashMap::new(),
            body: Vec::new()
        }
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn build(self) -> Result<Request, &'static str> {
        Ok(Request {
            method: self.method.ok_or("method missing")?,
            path: self.path.ok_or("path missing")?,
            version: self.version.unwrap_or_else(|| "HTTP/1.1".to_string()),
            headers: self.headers,
            body: self.body,
        })
    }

}

impl Request {
    /// Retrieves a header value by name (case-insensitive in HTTP practice).
    ///
    /// # Arguments
    ///
    /// * `key` - Header name to look up
    ///
    /// # Returns
    ///
    /// `Some(&str)` with the header value if present, `None` otherwise.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .get(key)
            .map(|v| v.as_str())
    }

    /// Retrieves the Content-Length header value and parses it as a usize.
    ///
    /// Returns 0 if the header is missing or not a valid number.
    pub fn content_length(&self) -> usize {
        self.header("Content-Length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    /// Determines whether the connection should remain open after the response.
    ///
    /// Checks the Connection header. For HTTP/1.1, the default is `true` (keep-alive).
    /// For HTTP/1.0 or if Connection: close is specified, returns `false`.
    pub fn keep_alive(&self) -> bool {
        self.header("Connection")
            .map(|v| v.eq_ignore_ascii_case("keep-alive"))
            .unwrap_or(true) // HTTP/1.1 default
    }
}
