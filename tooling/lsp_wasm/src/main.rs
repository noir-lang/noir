use async_lsp::{concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, server::LifecycleLayer};
use noir_lsp::NargoLspService;
use tower::ServiceBuilder;

mod backend;
mod stdio;

fn main() {
    // let blackbox_solver = acvm::blackbox_solver::BarretenbergSolver::initialize().await;
    let blackbox_solver = backend::MockBackend;
    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        let router = NargoLspService::new(&client, blackbox_solver);

        ServiceBuilder::new()
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .service(router)
    });

    let stdin = stdio::PipeStdin::lock().expect("stdin to lock");
    let stdout = stdio::PipeStdout::lock().expect("stdout to lock");

    futures::executor::block_on(async {
        server.run_buffered(stdin, stdout).await.expect("server should start");
    })
}
