use crate::i18n::i18n::t;
use crate::i18n::middleware::CURRENT_LANG;
use crate::resp::code::Code;
use crate::resp::err::E;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use validator::ValidationErrors;

#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> R<T> {
    pub fn ok(data: T) -> Self {
        let code = Code::Ok.as_i32();
        Self {
            code,
            message: translate(code, &vec![]),
            data: Some(data),
        }
    }

    pub fn err(err: E) -> Self {
        let (code, message) = map_err(err);
        Self {
            code,
            message,
            data: None,
        }
    }

    pub fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(err) => Self::err(err),
        }
    }
}


impl<T: Serialize> From<T> for R<T> {
    fn from(data: T) -> Self {
        Self::ok(data)
    }
}

impl<T: Serialize> From<E> for R<T> {
    fn from(err: E) -> Self {
        Self::err(err)
    }
}

impl R<()> {
    pub fn void() -> Self {
        Self::ok(())
    }

    pub fn from_unit(result: Result<(), E>) -> Self {
        match result {
            Ok(_) => Self::ok(()),
            Err(err) => Self::err(err),
        }
    }
}

impl<T: Serialize> IntoResponse for R<T> {
    fn into_response(self) -> axum::response::Response {
        let status = if self.code == Code::InternalServerError.as_i32() {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::OK
        };

        (status, Json(self)).into_response()
    }
}


fn map_err(err: E) -> (i32, String) {
    match err {
        E::Code(code) => (code, translate(code, &vec![])),
        E::Msg(code, params) => {
            let msg = translate(code, &params);
            (code, msg)
        }
        E::Sys(err) => {
            log::error!("{:?}", err);
            let code = Code::InternalServerError.as_i32();
            (code, translate(code, &vec![]))
        }
        E::Val(err) => {
            log::debug!("{:?}", err);
            let msg = format_validation_errors(&err);
            (Code::IllegalParam.as_i32(), msg)
        }
    }
}

fn format_validation_errors(err: &ValidationErrors) -> String {
    let mut msgs = Vec::new();
    for (field, errs) in err.field_errors() {
        for e in errs {
            let detail = match e.code.as_ref() {
                "required" => "is required".to_string(),
                "length" => {
                    let min = e.params.get("min");
                    let max = e.params.get("max");
                    match (min, max) {
                        (Some(min), Some(max)) => {
                            format!("length must be between {} and {}", min, max)
                        }
                        (Some(min), None) => format!("length must be at least {}", min),
                        (None, Some(max)) => format!("length must be at most {}", max),
                        _ => "length is invalid".to_string(),
                    }
                }
                "range" => {
                    let min = e.params.get("min");
                    let max = e.params.get("max");
                    match (min, max) {
                        (Some(min), Some(max)) => {
                            format!("must be between {} and {}", min, max)
                        }
                        (Some(min), None) => format!("must be at least {}", min),
                        (None, Some(max)) => format!("must be at most {}", max),
                        _ => "value is out of range".to_string(),
                    }
                }
                "email" => "must be a valid email".to_string(),
                _ => e
                    .message
                    .clone()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("invalid ({})", e.code)),
            };
            msgs.push(format!("{}: {}", field, detail));
        }
    }
    if msgs.is_empty() {
        translate(Code::IllegalParam.as_i32(), &vec![])
    } else {
        msgs.join("; ")
    }
}

fn translate(code: i32, params: &Vec<(String, String)>) -> String {
    let key = code.to_string();

    CURRENT_LANG
        .try_with(|lang| t(lang, &key, params))
        .unwrap_or_else(|_| t("zh", &key, params))
}

#[macro_export]
macro_rules! r {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => return $crate::resp::r::R::err(err.into()),
        }
    };
}
