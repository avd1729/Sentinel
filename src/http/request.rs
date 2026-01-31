use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct RequestBuilder {
    method: Option<Method>,
    path: Option<String>,
    version: Option<String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Method {
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
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .get(key)
            .map(|v| v.as_str())
    }

    pub fn content_length(&self) -> usize {
        self.header("Content-Length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    pub fn keep_alive(&self) -> bool {
        self.header("Connection")
            .map(|v| v.eq_ignore_ascii_case("keep-alive"))
            .unwrap_or(true) // HTTP/1.1 default
    }
}
