use super::*;
use crate::{ArcStr, app::config::mock::MockConfig, log::mock::MockLog};
use std::collections::HashMap;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string, header, method, path},
};

#[tokio::test]
async fn test_net_get_request_success() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("test response"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let result = net.get(url, None).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("test response"));
}

#[tokio::test]
async fn test_net_get_request_with_headers() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_string("json response"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let mut headers = HashMap::new();
    headers.insert(ArcStr::from("Accept"), ArcStr::from("application/json"));

    let url = ArcStr::from(&format!("{}/test", test_url));
    let result = net.get(url, Some(headers)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("json response"));
}

#[tokio::test]
async fn test_net_post_request_success() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/test"))
        .and(body_string("test body"))
        .respond_with(ResponseTemplate::new(200).set_body_string("posted"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let body = ArcStr::from("test body");
    let result = net.post(url, None, Some(body)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("posted"));
}

#[tokio::test]
async fn test_net_post_request_without_body() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("posted"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let result = net.post(url, None, None).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("posted"));
}

#[tokio::test]
async fn test_net_put_request_success() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/test"))
        .and(body_string("put body"))
        .respond_with(ResponseTemplate::new(200).set_body_string("updated"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let body = ArcStr::from("put body");
    let result = net.put(url, None, Some(body)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("updated"));
}

#[tokio::test]
async fn test_net_delete_request_success() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("DELETE"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("deleted"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let result = net.delete(url, None).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("deleted"));
}

#[tokio::test]
async fn test_net_patch_request_success() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("PATCH"))
        .and(path("/test"))
        .and(body_string("patch body"))
        .respond_with(ResponseTemplate::new(200).set_body_string("patched"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/test", test_url));
    let body = ArcStr::from("patch body");
    let result = net.patch(url, None, Some(body)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("patched"));
}

#[tokio::test]
async fn test_net_request_with_multiple_headers() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Accept", "application/json"))
        .and(header("Authorization", "Bearer token123"))
        .respond_with(ResponseTemplate::new(200).set_body_string("authenticated"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let mut headers = HashMap::new();
    headers.insert(ArcStr::from("Accept"), ArcStr::from("application/json"));
    headers.insert(
        ArcStr::from("Authorization"),
        ArcStr::from("Bearer token123"),
    );

    let url = ArcStr::from(&format!("{}/test", test_url));
    let result = net.get(url, Some(headers)).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("authenticated"));
}

#[tokio::test]
async fn test_net_request_error_handling() {
    let mock_server = MockServer::start().await;
    let test_url = ArcStr::from(&mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/notfound"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&mock_server)
        .await;

    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from(&format!("{}/notfound", test_url));
    let result = net.get(url, None).await;

    // Even with 404, the request succeeds but returns the error body
    // The actual error handling depends on the HTTP client behavior
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ArcStr::from("Not Found"));
}

#[tokio::test]
async fn test_net_request_invalid_url() {
    let mock_config = MockConfig::new();
    let mock_log = MockLog::new();
    let net = Net::spawn(mock_config, mock_log);

    let url = ArcStr::from("http://invalid-url-that-does-not-exist.local/");
    let result = net.get(url, None).await;

    // This should fail due to DNS resolution or connection error
    assert!(result.is_err());
}
