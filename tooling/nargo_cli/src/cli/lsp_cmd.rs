use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer,
};
use clap::Args;
use noir_lsp::NargoLspService;
use tower::ServiceBuilder;

use super::NargoConfig;
use crate::backends::Backend;
use crate::errors::CliError;

/// Starts the Noir LSP server
///
/// Starts an LSP server which allows IDEs such as VS Code to display diagnostics in Noir source.
///
/// VS Code Noir Language Support: https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir
#[derive(Debug, Clone, Args)]
pub(crate) struct LspCommand;

pub(crate) fn run(
    // Backend is currently unused, but we might want to use it to inform the lsp in the future
    _backend: &Backend,
    _args: LspCommand,
    _config: NargoConfig,
) -> Result<(), CliError> {
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let (server, _) = async_lsp::MainLoop::new_server(|client| {
            #[allow(deprecated)]
            let blackbox_solver = barretenberg_blackbox_solver::BarretenbergSolver::new();
            let router = NargoLspService::new(&client, blackbox_solver);

            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(LifecycleLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .layer(ClientProcessMonitorLayer::new(client))
                .service(router)
        });

        // Prefer truly asynchronous piped stdin/stdout without blocking tasks.
        #[cfg(unix)]
        let (stdin, stdout) = (
            async_lsp::stdio::PipeStdin::lock_tokio().unwrap(),
            async_lsp::stdio::PipeStdout::lock_tokio().unwrap(),
        );
        // Fallback to spawn blocking read/write otherwise.
        #[cfg(not(unix))]
        let (stdin, stdout) = (
            tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
            tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
        );

        server.run_buffered(stdin, stdout).await.map_err(CliError::LspError)
    })
}
