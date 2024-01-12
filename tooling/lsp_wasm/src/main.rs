#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use async_lsp::{concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, server::LifecycleLayer};
use noir_lsp::NargoLspService;
use tower::ServiceBuilder;

mod backend;

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

    let stdin = async_lsp::stdio::PipeStdin::lock().expect("stdin to lock");
    let stdout = async_lsp::stdio::PipeStdout::lock().expect("stdout to lock");

    let stdin = async_io::Async::new(stdin).expect("stdin to async-ify");
    let stdout = async_io::Async::new(stdout).expect("stdout to async-ify");

    futures::executor::block_on(async {
        server.run_buffered(stdin, stdout).await.expect("server should start");
    });
}
