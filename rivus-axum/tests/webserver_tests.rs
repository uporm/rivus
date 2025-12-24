use axum::{routing::get, Router};
use std::net::TcpListener;
use std::time::Duration;
use rivus_axum::result::Rerr;
use rivus_axum::WebServer;

#[tokio::test]
async fn test_i18n() {
    // Setup route that returns a 400 error
    let router = Router::new().route("/error", get(|| async { Rerr::Of(400) }));
    
    // Find a free port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener); // Free the port so WebServer can bind to it

    let addr_str = addr.to_string();
    let server = WebServer::new(router, addr_str.clone())
        .i18n_dir("tests/locales");
        
    // Run server in background
    tokio::spawn(async move {
        server.run().await.unwrap();
    });
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();

    // Test EN
    let resp = client
        .get(format!("http://{}/error", addr_str))
        .header("Accept-Language", "en")
        .send()
        .await
        .expect("Failed to send request");
    
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    println!("EN Response: {:?}", body);
    assert_eq!(body["message"], "Request Parameter Error");

    // Test ZH
    let resp = client
        .get(format!("http://{}/error", addr_str))
        .header("Accept-Language", "zh")
        .send()
        .await
        .expect("Failed to send request");
    
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    println!("ZH Response: {:?}", body);
    assert_eq!(body["message"], "请求参数错误");
}
