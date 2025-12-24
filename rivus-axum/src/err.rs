use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use validator::ValidationErrors;

#[allow(dead_code)]
#[derive(Error)]
pub enum Err {
    #[error("{0}")]
    Of(i32),
    #[error("{0}")]
    OfMessage(i32, HashMap<&'static str, String>),
    #[error("{0}")]
    System(#[from] anyhow::Error),
    #[error("{0}")]
    Validate(#[from] ValidationErrors),
}

impl fmt::Debug for Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Err::Of(code) => write!(f, "Error({:?})", code),
            Err::OfMessage(code, params) => write!(f, "Error({:?}, {:?})", code, params),
            Err::System(err) => write!(f, "{:?}", err),
            Err::Validate(err) => write!(f, "{:?}", err),
        }
    }
}
