//! YAML 配置加载器，支持环境变量替换

use dotenvy::dotenv;
use regex::Regex;
use serde::de::DeserializeOwned;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use thiserror::Error;

/// YAML 加载器错误
#[derive(Debug, Error)]
pub enum YamlLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    #[error("Invalid variable format: {0}")]
    InvalidVariable(String),
}

/// 替换 YAML 中的环境变量占位符
fn replace_vars(yaml_content: &str) -> Result<String, YamlLoaderError> {
    // 忽略 dotenv 加载错误（例如生产环境可能没有 .env 文件）
    let _ = dotenv();

    static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
    let re = VAR_REGEX.get_or_init(|| {
        Regex::new(r"\$\{([A-Z0-9_]+)(?::([^\}]*))?\}").expect("Invalid regex pattern")
    });

    let result = re.replace_all(yaml_content, |caps: &regex::Captures| {
        let var_name = &caps[1];
        let default = caps.get(2).map(|m| m.as_str());

        match env::var(var_name) {
            Ok(val) => val,
            Err(_) => default.unwrap_or("").to_string(),
        }
    });

    Ok(result.into_owned())
}

/// 从文件加载 YAML 配置
#[allow(dead_code)]
pub fn load_from_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, YamlLoaderError> {
    let content = fs::read_to_string(path)?;
    let replaced = replace_vars(&content)?;
    let data = serde_yaml::from_str(&replaced)?;
    Ok(data)
}

/// 从字符串加载 YAML 配置
#[allow(dead_code)]
pub fn load_from_str<T: DeserializeOwned>(yaml_content: &str) -> Result<T, YamlLoaderError> {
    let replaced = replace_vars(yaml_content)?;
    let data = serde_yaml::from_str(&replaced)?;
    Ok(data)
}

/// 编译时嵌入 YAML 文件
#[macro_export]
macro_rules! include_yaml {
    // 支持指定类型
    ($path:expr, $t:ty) => {
        $crate::yaml::load_from_str::<$t>(include_str!($path))
    };
}

pub use include_yaml;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_replace_vars_basic() {
        unsafe {
            env::set_var("TEST_VAR_BASIC", "basic_value");
        }
        let input = "key: ${TEST_VAR_BASIC}";
        let output = replace_vars(input).unwrap();
        assert_eq!(output, "key: basic_value");
    }

    #[test]
    fn test_replace_vars_default() {
        let input = "key: ${TEST_VAR_MISSING:default}";
        let output = replace_vars(input).unwrap();
        assert_eq!(output, "key: default");
    }

    #[test]
    fn test_replace_vars_no_default() {
        let input = "key: ${TEST_VAR_MISSING_NO_DEFAULT}";
        let output = replace_vars(input).unwrap();
        assert_eq!(output, "key: ");
    }

    #[test]
    fn test_load_from_str() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Config {
            host: String,
            port: u16,
        }

        unsafe {
            env::set_var("APP_HOST", "localhost");
        }
        let yaml = r#"
        host: ${APP_HOST}
        port: ${APP_PORT:8080}
        "#;

        let config: Config = load_from_str(yaml).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }
}
