use acvm::Backend;
use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, stdio::PipeStdin, tracing::TracingLayer,
};
use clap::Args;
use nargo_lsp::NargoLspService;
use noirc_driver::CompileOptions;
use tokio::io::BufReader;
use tower::ServiceBuilder;

use super::NargoConfig;
use crate::errors::CliError;

#[derive(Debug, Clone, Args)]
pub(crate) struct LspCommand {
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the lsp in the future
    _backend: &B,
    _args: LspCommand,
    _config: NargoConfig,
) -> Result<(), CliError<B>> {
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    let (server, _) = async_lsp::Frontend::new_server(|client| {
        let router = NargoLspService::new();

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client))
            .service(router)
    });

    runtime.block_on(async {
        let stdin = BufReader::new(PipeStdin::lock().unwrap());
        let stdout = async_lsp::stdio::PipeStdout::lock().unwrap();
        server.run(stdin, stdout).await.map_err(CliError::LspError)
    })
}
