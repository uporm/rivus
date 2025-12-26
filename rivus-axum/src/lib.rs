pub mod i18n;
pub use i18n::i18n::{I18nPart, internal_init_i18n};

pub mod resp;
pub mod server;
pub use server::WebServer;

// --- 修复点 1: 必须重导出 ctor，宏才能通过 rivus_axum::ctor 找到它 ---
pub use ctor;
