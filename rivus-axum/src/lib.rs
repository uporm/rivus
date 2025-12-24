use crate::i18n_middleware::handle_i18n;
use axum::middleware::from_fn;
use axum::{Router, middleware};
use axum::{extract::Request, middleware::Next, response::Response};
use std::future::Future;
use tokio::signal;

mod i18n_middleware;
pub mod result;
pub mod i18n;
pub mod r;
pub mod code;

pub struct WebServer {
    router: Router,
    address: String,
    i18n_dir: String,
}

impl WebServer {
    pub fn new(router: Router, address: impl Into<String>) -> Self {
        Self {
            router,
            address: address.into(),
            i18n_dir: "i18n".to_string(),
        }
    }

    pub fn i18n_dir(mut self, dir: impl Into<String>) -> Self {
        self.i18n_dir = dir.into();
        self.router = self.router.layer(from_fn(handle_i18n));
        self
    }

    pub fn with_middleware<F, Fut>(mut self, f: F) -> Self
    where
        F: Clone + Send + Sync + 'static + Fn(Request, Next) -> Fut,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router = self.router.layer(middleware::from_fn(f));
        self
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // 初始化 i18n
        i18n::init(&self.i18n_dir);

        tracing::info!("Starting web server at {}", self.address);

        let listener = tokio::net::TcpListener::bind(&self.address).await?;
        tracing::info!("⌛️ Waiting for connections...");
        tracing::info!("💡 Press Ctrl+C to stop the server");
        // 优雅关闭处理
        let server = axum::serve(listener, self.router).with_graceful_shutdown(shutdown_signal());
        if let Err(e) = server.await {
            tracing::error!("Server error: {}", e);
            return Err(anyhow::anyhow!("Server error: {}", e));
        }

        tracing::info!("Server shutdown completed");
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, starting graceful shutdown");
        },
    }
}
