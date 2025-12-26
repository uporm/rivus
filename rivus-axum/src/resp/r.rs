use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use crate::i18n::i18n::t;
use crate::i18n::middleware::CURRENT_LANG;
use crate::resp::code::Code;
use crate::resp::err::E;

#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> R<T> {
    pub fn ok(data: T) -> Self {
        let code = Code::Ok.as_i32();
        Self {
            code,
            message: message_of(code),
            data,
        }
    }
}

impl<T: Serialize + Default> R<T> {
    pub fn err(err: E) -> Self {
        let (code, message) = map_err(err);
        Self {
            code,
            message,
            data: T::default(),
        }
    }

    pub fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(err) => Self::err(err),
        }
    }
}

impl R<()> {
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
        E::Code(code) => (code, message_of(code)),
        E::Msg(code, params) => {
            let base = message_of(code);
            let msg = params
                .iter()
                .fold(base, |acc, (k, v)| acc.replace(&format!("{{{}}}", k), v));
            (code, msg)
        }
        E::Sys(err) => {
            log::error!("{:?}", err);
            let code = Code::InternalServerError.as_i32();
            (code, message_of(code))
        }
        E::Val(err) => {
            log::error!("{:?}", err);
            let is_missing = err
                .field_errors()
                .values()
                .any(|errs| errs.iter().any(|e| e.code == "required"));
            let code = if is_missing {
                Code::MissingParam.as_i32()
            } else {
                Code::IllegalParam.as_i32()
            };
            (code, message_of(code))
        }
    }
}

fn message_of(code: i32) -> String {
    let key = code.to_string();

    CURRENT_LANG
        .try_with(|lang| t(lang, &key, &[]))
        .ok()
        .unwrap_or(key)
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
