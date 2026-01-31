use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok,
    Created,
    NoContent,
    BadRequest,
    NotFound,
    MethodNotAllowed,
    InternalServerError
}

impl StatusCode {
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

pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct ResponseBuilder {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

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
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        ResponseBuilder::new(StatusCode::Ok)
            .body(body.into())
            .build()
    }

    pub fn not_found() -> Self {
        ResponseBuilder::new(StatusCode::NotFound)
            .body(b"404 Not Found".to_vec())
            .build()
    }

    pub fn internal_error() -> Self {
        ResponseBuilder::new(StatusCode::InternalServerError)
            .body(b"500 Internal Server Error".to_vec())
            .build()
    }
}


