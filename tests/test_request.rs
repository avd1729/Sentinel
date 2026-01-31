use sentinel::http::request::{Request, Method};
use std::collections::HashMap;

#[test]
fn test_request_header_retrieval() {
    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), "example.com".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert_eq!(req.header("Host"), Some("example.com"));
    assert_eq!(req.header("Content-Type"), Some("application/json"));
    assert_eq!(req.header("Missing"), None);
}

#[test]
fn test_request_content_length_parsing() {
    let mut headers = HashMap::new();
    headers.insert("Content-Length".to_string(), "42".to_string());

    let req = Request {
        method: Method::POST,
        path: "/api".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert_eq!(req.content_length(), 42);
}

#[test]
fn test_request_content_length_missing() {
    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: HashMap::new(),
        body: vec![],
    };

    assert_eq!(req.content_length(), 0);
}

#[test]
fn test_request_content_length_invalid() {
    let mut headers = HashMap::new();
    headers.insert("Content-Length".to_string(), "not-a-number".to_string());

    let req = Request {
        method: Method::POST,
        path: "/api".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert_eq!(req.content_length(), 0);
}

#[test]
fn test_request_keep_alive_http11_default() {
    // HTTP/1.1 defaults to keep-alive
    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: HashMap::new(),
        body: vec![],
    };

    assert!(req.keep_alive());
}

#[test]
fn test_request_keep_alive_explicit_header() {
    let mut headers = HashMap::new();
    headers.insert("Connection".to_string(), "keep-alive".to_string());

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert!(req.keep_alive());
}

#[test]
fn test_request_keep_alive_close() {
    let mut headers = HashMap::new();
    headers.insert("Connection".to_string(), "close".to_string());

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert!(!req.keep_alive());
}

#[test]
fn test_request_keep_alive_case_insensitive() {
    let mut headers = HashMap::new();
    headers.insert("Connection".to_string(), "Keep-Alive".to_string());

    let req = Request {
        method: Method::GET,
        path: "/".to_string(),
        version: "HTTP/1.1".to_string(),
        headers,
        body: vec![],
    };

    assert!(req.keep_alive());
}

#[test]
fn test_request_method_equality() {
    assert_eq!(Method::GET, Method::GET);
    assert_ne!(Method::GET, Method::POST);
}

#[test]
fn test_request_method_from_string() {
    assert_eq!(Method::from_str("GET"), Some(Method::GET));
    assert_eq!(Method::from_str("POST"), Some(Method::POST));
    assert_eq!(Method::from_str("INVALID"), None);
    assert_eq!(Method::from_str("get"), None); // Case-sensitive
}

#[test]
fn test_request_with_body() {
    let body_content = b"test body content".to_vec();
    let req = Request {
        method: Method::POST,
        path: "/api".to_string(),
        version: "HTTP/1.1".to_string(),
        headers: HashMap::new(),
        body: body_content.clone(),
    };

    assert_eq!(req.body, body_content);
}
