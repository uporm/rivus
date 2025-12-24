use serde::Serialize;
use std::collections::HashMap;
use crate::code::Code;

#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    #[serde(skip_serializing)]
    pub args: Option<HashMap<String, String>>,
}

impl<T: Serialize> R<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message: "ok".to_string(),
            data: Some(data),
            args: None,
        }
    }

    pub fn ok_with_message(data: T, message: String) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message,
            data: Some(data),
            args: None,
        }
    }

    pub fn err(code: i32) -> Self {
        Self {
            code,
            message: "error".to_string(),
            data: None,
            args: None,
        }
    }

    pub fn err_with_message(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
            args: None,
        }
    }

    pub fn err_with_args<K, V>(code: i32, args: HashMap<K, V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        let args = args
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self {
            code,
            message: "error".to_string(),
            data: None,
            args: Some(args),
        }
    }
}
