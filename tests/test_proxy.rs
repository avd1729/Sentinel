//! Tests for proxy upstream request handling

use sentinel::http::request::{Method, RequestBuilder};
use sentinel::proxy::backend::BackendPool;
use sentinel::proxy::upstream::ProxyHandler;
use std::time::Duration;

#[test]
fn test_build_http_request() {
    let handler = ProxyHandler::new(
        BackendPool::new(vec![]),
        Duration::from_secs(5),
        Duration::from_secs(30),
    );

    let request = RequestBuilder::new()
        .method(Method::GET)
        .path("/api/users")
        .version("HTTP/1.1")
        .header("User-Agent", "Test")
        .build()
        .unwrap();

    let backend_url = url::Url::parse("http://localhost:3000").unwrap();
    let request_bytes = handler.build_http_request(&request, &backend_url).unwrap();
    let request_str = String::from_utf8_lossy(&request_bytes);

    assert!(request_str.contains("GET /api/users HTTP/1.1"));
    assert!(request_str.contains("Host: localhost:3000"));
    assert!(request_str.contains("User-Agent: Test"));
    assert!(request_str.contains("Connection: close"));
}

#[test]
fn test_build_http_request_with_custom_port() {
    let handler = ProxyHandler::new(
        BackendPool::new(vec![]),
        Duration::from_secs(5),
        Duration::from_secs(30),
    );

    let request = RequestBuilder::new()
        .method(Method::POST)
        .path("/api/data")
        .version("HTTP/1.1")
        .header("Content-Type", "application/json")
        .build()
        .unwrap();

    let backend_url = url::Url::parse("http://localhost:8080").unwrap();
    let request_bytes = handler.build_http_request(&request, &backend_url).unwrap();
    let request_str = String::from_utf8_lossy(&request_bytes);

    assert!(request_str.contains("POST /api/data HTTP/1.1"));
    assert!(request_str.contains("Host: localhost:8080"));
    assert!(request_str.contains("Content-Type: application/json"));
}

#[test]
fn test_build_http_request_removes_hop_by_hop_headers() {
    let handler = ProxyHandler::new(
        BackendPool::new(vec![]),
        Duration::from_secs(5),
        Duration::from_secs(30),
    );

    let request = RequestBuilder::new()
        .method(Method::GET)
        .path("/")
        .version("HTTP/1.1")
        .header("Connection", "keep-alive")
        .header("Upgrade", "websocket")
        .header("User-Agent", "Test")
        .build()
        .unwrap();

    let backend_url = url::Url::parse("http://localhost:3000").unwrap();
    let request_bytes = handler.build_http_request(&request, &backend_url).unwrap();
    let request_str = String::from_utf8_lossy(&request_bytes);

    // Should have Connection: close (replaced)
    assert!(request_str.contains("Connection: close"));
    // Should NOT have Upgrade header (removed)
    assert!(!request_str.contains("Upgrade: websocket"));
    // Should still have User-Agent
    assert!(request_str.contains("User-Agent: Test"));
}

#[test]
fn test_build_http_request_default_path() {
    let handler = ProxyHandler::new(
        BackendPool::new(vec![]),
        Duration::from_secs(5),
        Duration::from_secs(30),
    );

    let request = RequestBuilder::new()
        .method(Method::GET)
        .path("")
        .version("HTTP/1.1")
        .build()
        .unwrap();

    let backend_url = url::Url::parse("http://localhost:3000").unwrap();
    let request_bytes = handler.build_http_request(&request, &backend_url).unwrap();
    let request_str = String::from_utf8_lossy(&request_bytes);

    // Empty path should default to "/"
    assert!(request_str.contains("GET / HTTP/1.1"));
}
