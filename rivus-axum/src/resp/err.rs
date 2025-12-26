use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error)]
pub enum E {
    #[error("{0}")]
    Code(i32),
    #[error("{0}")]
    Msg(i32, HashMap<&'static str, String>),
    #[error("{0}")]
    Sys(#[from] anyhow::Error),
    #[error("{0}")]
    Val(#[from] ValidationErrors),
}

impl fmt::Debug for E {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            E::Code(c) => write!(f, "E({c})"),
            E::Msg(c, p) => write!(f, "E({c}, {p:?})"),
            E::Sys(e) => write!(f, "{e:?}"),
            E::Val(e) => write!(f, "{e:?}"),
        }
    }
}
