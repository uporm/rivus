use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use rivus_axum::{r, WebServer};
use rivus_axum::code::Code;
use rivus_axum::err::Err;
use rivus_axum::i18n;
use rivus_axum::r::R;
use tower::ServiceExt;
use validator::Validate;

#[derive(serde::Serialize)]
struct Ping {
    pong: bool,
}

#[derive(serde::Serialize)]
struct Echo {
    name: String,
    age: i32,
}

#[derive(serde::Deserialize, Validate)]
struct EchoJson {
    #[validate(required, length(min = 1, max = 10))]
    name: Option<String>,
    #[validate(required)]
    age: Option<i32>,
}

async fn ping() -> R<Ping> {
    R::ok(Ping { pong: true })
}

async fn echo(Json(json): Json<EchoJson>) -> R<Option<Echo>> {
    r!(json.validate());

    R::ok(Some(Echo {
        name: json.name.unwrap(),
        age: json.age.unwrap(),
    }))
}

async fn bad_request() -> R<()> {
    R::<()>::err(Err::Of(Code::BadRequest.as_i32()))
}

async fn internal_error() -> R<()> {
    R::<()>::err(Err::System(anyhow::anyhow!("boom")))
}

fn locales_dir() -> String {
    format!("{}/tests/locales", env!("CARGO_MANIFEST_DIR"))
}

fn app() -> Router {
    Router::new()
        .route("/ping", get(ping))
        .route("/echo", post(echo))
        .route("/bad", get(bad_request))
        .route("/boom", get(internal_error))
}

async fn request(
    router: Router,
    method: &str,
    uri: &str,
    body: Body,
    content_type: Option<&str>,
    accept_language: Option<&str>,
) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);

    if let Some(v) = content_type {
        builder = builder.header("content-type", v);
    }

    if let Some(v) = accept_language {
        builder = builder.header("accept-language", v);
    }

    let response = router.oneshot(builder.body(body).unwrap()).await.unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

async fn request_get(
    router: Router,
    uri: &str,
    accept_language: Option<&str>,
) -> (StatusCode, String) {
    request(router, "GET", uri, Body::empty(), None, accept_language).await
}

async fn request_json(
    router: Router,
    uri: &str,
    json: &str,
    accept_language: Option<&str>,
) -> (StatusCode, String) {
    request(
        router,
        "POST",
        uri,
        Body::from(json.to_owned()),
        Some("application/json"),
        accept_language,
    )
    .await
}

#[tokio::test]
async fn ok_uses_accept_language_and_defaults_to_zh() {
    i18n::init(&locales_dir());

    let router = WebServer::new(app(), "127.0.0.1:0")
        .i18n_dir(locales_dir())
        .into_router();

    let (status_en, body_en) = request_get(router.clone(), "/ping", Some("EN-US,en;q=0.9")).await;
    assert_eq!(status_en, StatusCode::OK);
    assert!(body_en.contains("\"code\":200"));
    assert!(body_en.contains("\"message\":\"Ok\""));
    assert!(body_en.contains("\"data\":{\"pong\":true}"));

    let (status_zh, body_zh) = request_get(router.clone(), "/ping", None).await;
    assert_eq!(status_zh, StatusCode::OK);
    assert!(body_zh.contains("\"code\":200"));
    assert!(body_zh.contains("\"message\":\"成功\""));
    assert!(body_zh.contains("\"data\":{\"pong\":true}"));
}

#[tokio::test]
async fn non_500_business_errors_keep_http_200_and_are_localized() {
    i18n::init(&locales_dir());

    let router = WebServer::new(app(), "127.0.0.1:0")
        .i18n_dir(locales_dir())
        .into_router();

    let (status_en, body_en) = request_get(router.clone(), "/bad", Some("en")).await;
    assert_eq!(status_en, StatusCode::OK);
    assert!(body_en.contains("\"code\":400"));
    assert!(body_en.contains("\"message\":\"Request Parameter Error\""));
    assert!(body_en.contains("\"data\":null"));

    let (status_zh, body_zh) = request_get(router.clone(), "/bad", Some("zh-CN,zh;q=0.9")).await;
    assert_eq!(status_zh, StatusCode::OK);
    assert!(body_zh.contains("\"code\":400"));
    assert!(body_zh.contains("\"message\":\"请求参数错误\""));
    assert!(body_zh.contains("\"data\":null"));
}

#[tokio::test]
async fn json_params_are_validated_and_return_business_codes() {
    i18n::init(&locales_dir());

    let router = WebServer::new(app(), "127.0.0.1:0")
        .i18n_dir(locales_dir())
        .into_router();

    let (status_ok, body_ok) = request_json(
        router.clone(),
        "/echo",
        r#"{"name":"jack","age":18}"#,
        Some("en"),
    )
    .await;
    assert_eq!(status_ok, StatusCode::OK);
    assert!(body_ok.contains("\"code\":200"));
    assert!(body_ok.contains("\"message\":\"Ok\""));
    assert!(body_ok.contains("\"data\":{\"name\":\"jack\",\"age\":18}"));

    let (status_missing, body_missing) =
        request_json(router.clone(), "/echo", r#"{"name":"jack"}"#, Some("zh")).await;
    assert_eq!(status_missing, StatusCode::OK);
    assert!(body_missing.contains("\"code\":901"));
    assert!(body_missing.contains("\"message\":\"缺少必要参数\""));
    assert!(body_missing.contains("\"data\":null"));

    let (status_illegal, body_illegal) = request_json(
        router.clone(),
        "/echo",
        r#"{"name":"","age":18}"#,
        Some("en"),
    )
    .await;
    assert_eq!(status_illegal, StatusCode::OK);
    assert!(body_illegal.contains("\"code\":902"));
    assert!(body_illegal.contains("\"message\":\"Illegal Parameter\""));
    assert!(body_illegal.contains("\"data\":null"));

    let (status_parse, _body_parse) = request_json(
        router,
        "/echo",
        r#"{"name":"jack","age":"abc"}"#,
        Some("zh-CN"),
    )
    .await;
    // Axum returns 422 for deserialization errors by default
    assert_eq!(status_parse, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn internal_errors_map_to_http_500_and_are_localized() {
    i18n::init(&locales_dir());

    let router = WebServer::new(app(), "127.0.0.1:0")
        .i18n_dir(locales_dir())
        .into_router();

    let (status, body) = request_get(router, "/boom", Some("en")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body.contains("\"code\":500"));
    assert!(body.contains("\"message\":\"Internal Server Error\""));
    assert!(body.contains("\"data\":null"));
}

#[tokio::test]
async fn unknown_route_returns_http_404_from_axum() {
    i18n::init(&locales_dir());

    let router = WebServer::new(app(), "127.0.0.1:0")
        .i18n_dir(locales_dir())
        .into_router();

    let (status, _body) = request_get(router, "/missing", Some("en")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
