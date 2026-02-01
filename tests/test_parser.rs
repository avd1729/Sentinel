use sentinel::http::parser::{ParseError, parse_http_request};
use sentinel::http::request::Method;

#[test]
fn test_parse_simple_get_request() {
    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let (parsed, consumed) = parse_http_request(req).unwrap();

    assert_eq!(parsed.method, Method::GET);
    assert_eq!(parsed.path, "/");
    assert_eq!(parsed.version, "HTTP/1.1");
    assert_eq!(parsed.headers.get("Host").unwrap(), "example.com");
    assert_eq!(consumed, req.len());
}

#[test]
fn test_parse_post_request_with_body() {
    let req = b"POST /api HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\nhello";
    let (parsed, consumed) = parse_http_request(req).unwrap();

    assert_eq!(parsed.method, Method::POST);
    assert_eq!(parsed.path, "/api");
    assert_eq!(parsed.version, "HTTP/1.1");
    assert_eq!(parsed.body, b"hello".to_vec());
    assert_eq!(consumed, req.len());
}

#[test]
fn test_parse_multiple_headers() {
    let req = b"GET /path HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test-client\r\nAccept: */*\r\n\r\n";
    let (parsed, _) = parse_http_request(req).unwrap();

    assert_eq!(parsed.headers.get("Host").unwrap(), "example.com");
    assert_eq!(parsed.headers.get("User-Agent").unwrap(), "test-client");
    assert_eq!(parsed.headers.get("Accept").unwrap(), "*/*");
}

#[test]
fn test_parse_request_with_path_and_query_string() {
    let req = b"GET /search?q=rust HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let (parsed, _) = parse_http_request(req).unwrap();

    assert_eq!(parsed.path, "/search?q=rust");
}

#[test]
fn test_parse_incomplete_request_missing_blank_line() {
    let req = b"GET / HTTP/1.1\r\nHost: example.com\r\n";
    let result = parse_http_request(req);

    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn test_parse_incomplete_request_partial_body() {
    let req = b"POST /api HTTP/1.1\r\nContent-Length: 10\r\n\r\nhello";
    let result = parse_http_request(req);

    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn test_parse_invalid_http_method() {
    let req = b"INVALID / HTTP/1.1\r\n\r\n";
    let result = parse_http_request(req);

    assert!(matches!(result, Err(ParseError::InvalidMethod)));
}

#[test]
fn test_parse_malformed_header() {
    let req = b"GET / HTTP/1.1\r\nBrokenHeader\r\n\r\n";
    let result = parse_http_request(req);

    assert!(matches!(result, Err(ParseError::InvalidHeader)));
}

#[test]
fn test_parse_various_http_methods() {
    let methods = vec![
        ("GET", Method::GET),
        ("POST", Method::POST),
        ("PUT", Method::PUT),
        ("DELETE", Method::DELETE),
        ("HEAD", Method::HEAD),
        ("OPTIONS", Method::OPTIONS),
        ("PATCH", Method::PATCH),
    ];

    for (method_str, expected_method) in methods {
        let req = format!("{} / HTTP/1.1\r\n\r\n", method_str);
        let (parsed, _) = parse_http_request(req.as_bytes()).unwrap();
        assert_eq!(parsed.method, expected_method);
    }
}

#[test]
fn test_parse_request_with_empty_body() {
    let req = b"POST /api HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
    let (parsed, _) = parse_http_request(req).unwrap();

    assert_eq!(parsed.body.len(), 0);
}

#[test]
fn test_parse_request_with_binary_body() {
    let req = b"POST /upload HTTP/1.1\r\nContent-Length: 4\r\n\r\n\x00\x01\x02\x03";
    let (parsed, _) = parse_http_request(req).unwrap();

    assert_eq!(parsed.body, vec![0, 1, 2, 3]);
}

#[test]
fn test_parse_header_case_preservation() {
    let req = b"GET / HTTP/1.1\r\nContent-Type: application/json\r\n\r\n";
    let (parsed, _) = parse_http_request(req).unwrap();

    // Headers are stored as-is with trimming
    assert!(parsed.headers.contains_key("Content-Type"));
}
