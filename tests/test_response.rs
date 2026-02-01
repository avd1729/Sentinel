use sentinel::http::response::{Response, ResponseBuilder, StatusCode};

#[test]
fn test_status_code_as_u16() {
    assert_eq!(StatusCode::Ok.as_u16(), 200);
    assert_eq!(StatusCode::Created.as_u16(), 201);
    assert_eq!(StatusCode::NoContent.as_u16(), 204);
    assert_eq!(StatusCode::BadRequest.as_u16(), 400);
    assert_eq!(StatusCode::NotFound.as_u16(), 404);
    assert_eq!(StatusCode::MethodNotAllowed.as_u16(), 405);
    assert_eq!(StatusCode::InternalServerError.as_u16(), 500);
}

#[test]
fn test_status_code_reason_phrase() {
    assert_eq!(StatusCode::Ok.reason_phrase(), "OK");
    assert_eq!(StatusCode::Created.reason_phrase(), "Created");
    assert_eq!(StatusCode::NoContent.reason_phrase(), "No Content");
    assert_eq!(StatusCode::BadRequest.reason_phrase(), "Bad Request");
    assert_eq!(StatusCode::NotFound.reason_phrase(), "Not Found");
    assert_eq!(
        StatusCode::MethodNotAllowed.reason_phrase(),
        "Method Not Allowed"
    );
    assert_eq!(
        StatusCode::InternalServerError.reason_phrase(),
        "Internal Server Error"
    );
}

#[test]
fn test_response_builder_basic() {
    let response = ResponseBuilder::new(StatusCode::Ok)
        .body(b"Hello, World!".to_vec())
        .build();

    assert_eq!(response.status, StatusCode::Ok);
    assert_eq!(response.body, b"Hello, World!".to_vec());
}

#[test]
fn test_response_builder_with_headers() {
    let response = ResponseBuilder::new(StatusCode::Ok)
        .header("Content-Type", "text/plain")
        .header("X-Custom", "value")
        .body(b"test".to_vec())
        .build();

    assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");
    assert_eq!(response.headers.get("X-Custom").unwrap(), "value");
}

#[test]
fn test_response_builder_auto_content_length() {
    let body = b"This is the body".to_vec();
    let response = ResponseBuilder::new(StatusCode::Ok)
        .body(body.clone())
        .build();

    let content_length = response.headers.get("Content-Length").unwrap();
    assert_eq!(content_length, &body.len().to_string());
}

#[test]
fn test_response_builder_preserves_custom_content_length() {
    let response = ResponseBuilder::new(StatusCode::Ok)
        .header("Content-Length", "999")
        .body(b"test".to_vec())
        .build();

    // Should keep the custom value
    assert_eq!(response.headers.get("Content-Length").unwrap(), "999");
}

#[test]
fn test_response_builder_multiple_headers() {
    let response = ResponseBuilder::new(StatusCode::Ok)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-cache")
        .header("X-Frame-Options", "DENY")
        .body(b"{}".to_vec())
        .build();

    assert_eq!(response.headers.len(), 4); // 3 custom + 1 auto (Content-Length)
    assert_eq!(
        response.headers.get("Content-Type").unwrap(),
        "application/json"
    );
    assert_eq!(response.headers.get("Cache-Control").unwrap(), "no-cache");
    assert_eq!(response.headers.get("X-Frame-Options").unwrap(), "DENY");
}

#[test]
fn test_response_builder_empty_body() {
    let response = ResponseBuilder::new(StatusCode::NoContent).build();

    assert_eq!(response.body.len(), 0);
    assert_eq!(response.headers.get("Content-Length").unwrap(), "0");
}

#[test]
fn test_response_builder_various_status_codes() {
    let statuses = vec![
        StatusCode::Ok,
        StatusCode::Created,
        StatusCode::BadRequest,
        StatusCode::NotFound,
    ];

    for status in statuses {
        let response = ResponseBuilder::new(status).body(b"test".to_vec()).build();
        assert_eq!(response.status, status);
    }
}

#[test]
fn test_response_builder_fluent_api() {
    // Test that builder methods return self for chaining
    let response = ResponseBuilder::new(StatusCode::Ok)
        .header("Header1", "value1")
        .header("Header2", "value2")
        .header("Header3", "value3")
        .body(b"body".to_vec())
        .build();

    assert_eq!(response.headers.len(), 4); // 3 custom + 1 auto
}

#[test]
fn test_response_ok_helper() {
    let response = Response::ok(b"test content".to_vec());

    assert_eq!(response.status, StatusCode::Ok);
    assert_eq!(response.body, b"test content".to_vec());
}

#[test]
fn test_response_not_found_helper() {
    let response = Response::not_found();

    assert_eq!(response.status, StatusCode::NotFound);
    assert_eq!(response.body, b"404 Not Found".to_vec());
}

#[test]
fn test_response_internal_error_helper() {
    let response = Response::internal_error();

    assert_eq!(response.status, StatusCode::InternalServerError);
    assert_eq!(response.body, b"500 Internal Server Error".to_vec());
}
