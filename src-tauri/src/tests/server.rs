use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::tests::runtime_generator::TestContext;

#[derive(Clone)]
pub struct TestServer {
    pub addr: SocketAddr,
    pub base_url: String,
}

impl TestServer {
    pub async fn start(ctx: &TestContext) -> Result<Self, String> {
        let assets_root = ctx.assets_dir();
        let libraries_root = ctx.libraries_dir();

        let app = Router::new()
            .nest_service("/assets", ServeDir::new(assets_root))
            .nest_service("/libraries", ServeDir::new(libraries_root));

        let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|e| e.to_string())?;

        let addr = listener.local_addr().map_err(|e| e.to_string())?;

        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        Ok(Self { addr, base_url })
    }

    pub fn assets_url(&self, path: impl AsRef<str>) -> String {
        let p = path.as_ref().trim_start_matches('/');
        format!("{}/assets/{}", self.base_url, p)
    }

    pub fn libraries_url(&self, path: impl AsRef<str>) -> String {
        let p = path.as_ref().trim_start_matches('/');
        format!("{}/libraries/{}", self.base_url, p)
    }

    pub fn asset_index_url(&self, id: &str) -> String {
        format!("{}/assets/indexes/{}.json", self.base_url, id)
    }

    pub fn library_url(&self, maven_path: &str) -> String {
        let p = maven_path.trim_start_matches('/');
        format!("{}/libraries/{}", self.base_url, p)
    }
}
