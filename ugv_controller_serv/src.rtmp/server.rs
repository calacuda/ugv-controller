use log::*;
use rtmp_rs::protocol::message::{ConnectParams, PublishParams};
use rtmp_rs::session::SessionContext;
use rtmp_rs::{AuthResult, RtmpHandler, RtmpServer, ServerConfig};

struct MyHandler;

impl RtmpHandler for MyHandler {
    async fn on_connect(&self, _ctx: &SessionContext, params: &ConnectParams) -> AuthResult {
        debug!("App: {}", params.app);
        AuthResult::Accept
    }

    async fn on_publish(&self, _ctx: &SessionContext, params: &PublishParams) -> AuthResult {
        debug!("Stream key: {}", params.stream_key);
        // Add your stream key validation here
        AuthResult::Accept
    }

    // See RtmpHandler trait for other available callbacks
}

#[tokio::main]
pub async fn server() -> Result<(), Box<dyn std::error::Error>> {
    info!("starting...");
    let server = RtmpServer::new(ServerConfig::default(), MyHandler);
    server.run().await?;

    Ok(())
}
