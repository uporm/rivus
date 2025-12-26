use std::collections::HashMap;
use std::sync::OnceLock;

pub use ctor; // 重产出给宏使用

#[derive(Debug, Clone)]
pub enum I18nPart {
    Static(&'static str),
    Placeholder(&'static str),
}

// 存储结构：Map<语言, Map<键, 片段集合>>
pub type I18nMap = HashMap<&'static str, HashMap<&'static str, Vec<I18nPart>>>;

static I18N_STORAGE: OnceLock<I18nMap> = OnceLock::new();

/// 宏调用的内部初始化接口
pub fn internal_init_i18n(data: I18nMap) {
    let _ = I18N_STORAGE.set(data);
}

/// 生产级翻译函数
pub fn t(lang: &str, key: &str, args: &[(&str, &str)]) -> String {
    let Some(lang_map) = I18N_STORAGE.get().and_then(|m| m.get(lang)) else {
        return format!("[Missing Lang: {}]", lang);
    };

    let Some(parts) = lang_map.get(key) else {
        return format!("[Missing Key: {}]", key);
    };

    // 预分配内存提高性能
    let mut result = String::with_capacity(128);
    for part in parts {
        match part {
            I18nPart::Static(s) => result.push_str(s),
            I18nPart::Placeholder(p_name) => {
                let val = args.iter()
                    .find(|(k, _)| k == p_name)
                    .map(|(_, v)| *v)
                    .unwrap_or("");
                result.push_str(val);
            }
        }
    }
    result
}