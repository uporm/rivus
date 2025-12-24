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
    let default_lang = "zh";

    let Some(store) = I18N_STORE.get() else {
        return default_lang.to_string();
    };

    let Some(header) = req
        .headers()
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
    else {
        return default_lang.to_string();
    };

    for raw in header.split(',') {
        let tag = raw.split(';').next().unwrap_or(raw).trim().to_lowercase();

        if tag.is_empty() {
            continue;
        }

        if store.contains_key(&tag) {
            return tag;
        }

        if let Some(primary) = tag.split('-').next()
            && store.contains_key(primary)
        {
            return primary.to_string();
        }
    }

    default_lang.to_string()
}
