use acvm::Backend;
use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer,
};
use clap::Args;
use noir_lsp::NargoLspService;
use tokio::io::BufReader;
use tower::ServiceBuilder;

use super::NargoConfig;
use crate::errors::CliError;

#[derive(Debug, Clone, Args)]
pub(crate) struct LspCommand;

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the lsp in the future
    _backend: &B,
    _args: LspCommand,
    _config: NargoConfig,
) -> Result<(), CliError<B>> {
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let (server, _) = async_lsp::Frontend::new_server(|client| {
            let router = NargoLspService::new(&client);

            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(LifecycleLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .layer(ClientProcessMonitorLayer::new(client))
                .service(router)
        });

        // Prefer truely asynchronous piped stdin/stdout without blocking tasks.
        #[cfg(unix)]
        let (stdin, stdout) = (
            async_lsp::stdio::PipeStdin::lock_tokio().unwrap(),
            async_lsp::stdio::PipeStdout::lock_tokio().unwrap(),
        );
        // Fallback to spawn blocking read/write otherwise.
        #[cfg(not(unix))]
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

        let stdin = BufReader::new(stdin);

        server.run(stdin, stdout).await.map_err(CliError::LspError)
    })
}
