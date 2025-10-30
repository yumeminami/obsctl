use obsctl::{config::AppContext, mcp::ObsctlMcpServer};
use rmcp::{
    service::{QuitReason, ServiceExt},
    transport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AppContext::load()?;
    let service = ObsctlMcpServer::new(ctx);
    let transport = transport::stdio();

    let running = service.serve(transport).await?;
    match running.waiting().await {
        Ok(QuitReason::JoinError(err)) => Err(err.into()),
        Err(err) => Err(err.into()),
        _ => Ok(()),
    }
}
