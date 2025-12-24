use axum::Json;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use validator::ValidationErrors;
use crate::code::Code;
use crate::i18n;
use crate::i18n::CURRENT_LANG;
use crate::r::R;

pub struct Rok<T>(pub T);

impl<T: Serialize> IntoResponse for Rok<T> {
    fn into_response(self) -> Response<Body> {
        let lang = CURRENT_LANG.with(|lang| lang.clone());
        let msg = i18n::translate(&lang, &Code::Ok.to_string()).unwrap_or_else(|| Code::Ok.to_string());

        let r = R::ok_with_message(Some(self.0), msg);
        (StatusCode::OK, Json(r)).into_response()
    }
}

#[allow(dead_code)]
#[derive(Error)]
pub enum Rerr {
    #[error("{0}")]
    Of(i32),
    #[error("{0}")]
    OfMessage(i32, HashMap<&'static str, String>),
    #[error("{0}")]
    Validate(#[from] ValidationErrors),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl fmt::Debug for Rerr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rerr::Of(code) => write!(f, "Error({:?})", code),
            Rerr::OfMessage(code, params) => write!(f, "Error({:?}, {:?})", code, params),
            Rerr::Validate(err) => write!(f, "ValidationError({:?})", err),
            Rerr::Other(err) => write!(f, "{:?}", err),
        }
    }
}

impl IntoResponse for Rerr {
    fn into_response(self) -> Response<Body> {
        let (status, r) = match self {
            Rerr::Of(code) => {
                let lang = CURRENT_LANG.with(|lang| lang.clone());
                let msg = i18n::translate(&lang, &code.to_string()).unwrap_or_else(|| code.to_string());
                (
                    StatusCode::OK,
                    R::<()>::err_with_message(code, msg),
                )
            }
            Rerr::OfMessage(code, params) => {
                // 从 task-local 读取语言
                let lang = CURRENT_LANG.with(|lang| lang.clone());
                let mut msg = i18n::translate(&lang, &code.to_string()).unwrap_or_else(|| code.to_string());
                
                for (k, v) in &params {
                    msg = msg.replace(&format!("{{{}}}", k), v);
                }

                (
                    StatusCode::OK,
                    R::<()>::err_with_message(code, msg),
                )
            },
            Rerr::Validate(e) => (
                StatusCode::BAD_REQUEST,
                R::err_with_message(Code::BadRequest.as_i32(), e.to_string()),
            ),
            Rerr::Other(_) => {
                tracing::error!("{:?}", self);
                let lang = CURRENT_LANG.with(|lang| lang.clone());
                let msg = i18n::translate(&lang, &Code::InternalServerError.to_string()).unwrap_or_else(|| Code::InternalServerError.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    R::<()>::err_with_message(Code::InternalServerError.as_i32(), msg),
                )
            }
        };

        (status, Json(r)).into_response()
    }
}