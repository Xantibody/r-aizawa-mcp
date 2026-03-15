use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
struct FetchParams {
    /// The URL to fetch content from
    url: String,
}

#[derive(Clone)]
struct RaizawaMcp {
    client: reqwest::Client,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RaizawaMcp {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Fetch content from a URL and return it as text")]
    async fn fetch(&self, params: Parameters<FetchParams>) -> Result<CallToolResult, McpError> {
        let url = &params.0.url;
        let response = self.client.get(url).send().await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to fetch URL: {e}").into(),
            data: None,
        })?;

        let status = response.status();
        if !status.is_success() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "HTTP error: {status}"
            ))]));
        }

        let body = response.text().await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to read response body: {e}").into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(body)]))
    }
}

#[tool_handler]
impl ServerHandler for RaizawaMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
            .with_server_info(Implementation::new(
                "r-aizawa-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions("A personal MCP server for fetching URL content")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let server = RaizawaMcp::new();
    let transport = rmcp::transport::io::stdio();
    server.serve(transport).await?;

    Ok(())
}
