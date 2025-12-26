// i18n/middleware.rs
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tokio::task_local;

task_local! {
    pub static CURRENT_LANG: String;
}

pub async fn handle_i18n(req: Request, next: Next) -> Response {
    let lang = resolve_language(&req);
    CURRENT_LANG.scope(lang, next.run(req)).await
}

fn resolve_language(req: &Request) -> String {
    let default_lang = "zh-CN".to_string();

    let Some(header) = req
        .headers()
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

    langs.first().map(|l| l.1.clone()).unwrap_or(default_lang)
}
