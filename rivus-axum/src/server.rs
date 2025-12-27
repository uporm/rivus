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
    middlewares: Vec<Box<dyn FnOnce(Router) -> Router + Send>>,
}

impl WebServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            router: Router::new(),
            addr: addr.into(),
            middlewares: Vec::new(),
        }
    }

    pub fn layer_i18n(mut self) -> Self {
        self.middlewares.push(Box::new(|r| r.layer(from_fn(handle_i18n))));
        self
    }

    pub fn layer_fn<F, Fut>(mut self, f: F) -> Self
    where
        F: Clone + Send + Sync + 'static + Fn(Request, Next) -> Fut,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.middlewares.push(Box::new(|r| r.layer(middleware::from_fn(f))));
        self
    }

    pub fn mount(mut self, router: Router) -> Self {
        self.router = self.router.merge(router);
        self
    }

    pub async fn start(mut self) -> anyhow::Result<()> {
        log::info!("🚀 Starting web server at {}", self.addr);

        for m in self.middlewares {
            self.router = m(self.router);
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, routing::get};
    use http::Request;
    use tower::util::ServiceExt; 
    use crate::i18n::middleware::CURRENT_LANG;

    async fn check_lang() -> String {
        CURRENT_LANG.try_with(|l| l.clone()).unwrap_or_else(|_| "not set".to_string())
    }

    #[tokio::test]
    async fn test_layer_ordering_success() {
        // Case 2: mount then layer
        let server = WebServer::new("0.0.0.0:0")
            .mount(Router::new().route("/", get(check_lang)))
            .layer_i18n();
        
        // Simulate start() logic to apply middlewares
        let mut router = server.router;
        for m in server.middlewares {
            router = m(router);
        }

        let app = router;
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = ServiceExt::oneshot(app, req).await.unwrap();
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Here it should be set (default "zh-CN")
        assert_eq!(body_str, "zh-CN");
    }

    #[tokio::test]
    async fn test_layer_ordering_deferred() {
        // Case 1: layer then mount (Used to fail, now should succeed)
        let server = WebServer::new("0.0.0.0:0")
            .layer_i18n()
            .mount(Router::new().route("/", get(check_lang)));
        
        // Simulate start() logic
        let mut router = server.router;
        for m in server.middlewares {
            router = m(router);
        }

        let app = router;
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = ServiceExt::oneshot(app, req).await.unwrap();
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Now it should be set because layers are applied at the end
        assert_eq!(body_str, "zh-CN");
    }
}
