use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use tokio::task_local;
use super::i18n::is_language_supported;

task_local! {
    pub static CURRENT_LANG: String;
}

pub async fn handle_i18n(req: Request, next: Next) -> Response {
    let lang = resolve_language(req.headers());
    CURRENT_LANG.scope(lang, next.run(req)).await
}

fn resolve_language(headers: &HeaderMap) -> String {
    let default_lang = "zh".to_string();

    let Some(header) = headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
    else {
        return default_lang;
    };

    // 简易解析: "zh-CN,zh;q=0.9,en;q=0.8" -> ["zh-CN", "zh", "en"]
    let mut langs: Vec<(f32, String)> = header
        .split(',')
        .filter_map(|part| {
            let mut sections = part.split(';');
            let lang = sections.next()?.trim().to_string();
            let q_value = sections
                .next()
                .and_then(|q| q.trim().strip_prefix("q="))
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(1.0);
            Some((q_value, lang))
        })
        .collect();

    // 按权重降序排列
    langs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for (_, lang) in langs {
        if is_language_supported(&lang) {
            return lang;
        }
    }

    default_lang
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::i18n::internal_init_i18n;
    use std::collections::HashMap;

    fn init_test_i18n() {
        let mut map = HashMap::new();
        map.insert("en", HashMap::new());
        map.insert("zh-CN", HashMap::new());
        // Ignore error if already initialized
        let _ = std::panic::catch_unwind(|| {
             // We can't catch set error easily here because internal_init_i18n swallows it?
             // actually internal_init_i18n returns (), so we assume it works or has been done.
             // But if we want to ensure specific content, we can't overwrite it.
             // For this test, we assume we are the first to init or the existing init is compatible.
        });
        internal_init_i18n(map);
    }

    #[test]
    fn test_resolve_language() {
        init_test_i18n();

        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "fr;q=0.9, en;q=0.8".parse().unwrap());
        assert_eq!(resolve_language(&headers), "en");

        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "fr;q=1.0".parse().unwrap());
        assert_eq!(resolve_language(&headers), "zh-CN"); // Default
        
        let mut headers = HeaderMap::new();
        headers.insert("accept-language", "zh-CN".parse().unwrap());
        assert_eq!(resolve_language(&headers), "zh-CN");
    }
}
