use crate::i18n::{CURRENT_LANG, I18N_STORE};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn handle_i18n(req: Request, next: Next) -> Response {
    let lang = resolve_language(&req);

    // 在当前 task 中设置语言
    CURRENT_LANG
        .scope(lang, async move { next.run(req).await })
        .await
}

fn resolve_language(req: &Request) -> String {
    req.headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .into_iter()
        .flat_map(|v| v.split(','))
        .map(|s| s.split(';').next().unwrap_or(s).trim())
        .map(|s| s.to_lowercase())
        .find(|lang| is_lang_supported(lang))
        .unwrap_or_else(|| "zh".to_string())
}

fn is_lang_supported(lang: &str) -> bool {
    I18N_STORE
        .get()
        .map(|store| store.contains_key(lang))
        .unwrap_or(false)
}
