use std::future::Future;

use axum::extract::Request;
use axum::middleware::{Next, from_fn};
use axum::response::Response;
use axum::{Router, middleware};
use tokio::signal;

use crate::i18n::middleware::handle_i18n;

pub struct WebServer {
    router: Router,
    addr: String,
}

impl WebServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            router: Router::new(),
            addr: addr.into(),
        }
    }

    pub fn layer_i18n(mut self) -> Self {
        self.router = self.router.layer(from_fn(handle_i18n));
        self
    }

    pub fn layer_fn<F, Fut>(mut self, f: F) -> Self
    where
        F: Clone + Send + Sync + 'static + Fn(Request, Next) -> Fut,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.router = self.router.layer(middleware::from_fn(f));
        self
    }

    pub fn mount(mut self, router: Router) -> Self {
        self.router = self.router.merge(router);
        self
    }

    pub async fn start(self) -> anyhow::Result<()> {
        log::info!("🚀 Starting web server at {}", self.addr);

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        // 优雅关闭处理
        let server = axum::serve(listener, self.router).with_graceful_shutdown(wait_for_shutdown());
        if let Err(e) = server.await {
            log::error!("Server error: {}", e);
            return Err(anyhow::anyhow!("Server error: {}", e));
        }

        log::info!("🛑 Server stopped");
        Ok(())
    }
}

async fn wait_for_shutdown() {
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
            log::info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            log::info!("Received terminate signal, starting graceful shutdown");
        },
    }
}
