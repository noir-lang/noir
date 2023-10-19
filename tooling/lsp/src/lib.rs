#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use std::{
    future::Future,
    ops::{self, ControlFlow},
    path::{Path, PathBuf},
    pin::Pin,
    task::{self, Poll},
};

use acvm::BlackBoxFunctionSolver;
use async_lsp::{
    router::Router, AnyEvent, AnyNotification, AnyRequest, ClientSocket, Error, LspService,
    ResponseError,
};
use codespan_reporting::files;
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{Context, FunctionNameMatch},
};
use notifications::{
    on_did_change_configuration, on_did_change_text_document, on_did_close_text_document,
    on_did_open_text_document, on_did_save_text_document, on_exit, on_initialized,
};
use requests::{
    on_code_lens_request, on_initialize, on_shutdown, on_test_run_request, on_tests_request,
};
use serde_json::Value as JsonValue;
use tower::Service;

mod notifications;
mod requests;
mod solver;
mod types;

use solver::WrapperSolver;
use types::{notification, request, NargoTest, NargoTestId, Position, Range, Url};

// State for the LSP gets implemented on this struct and is internal to the implementation
pub struct LspState {
    root_path: Option<PathBuf>,
    client: ClientSocket,
    solver: WrapperSolver,
}

impl LspState {
    fn new(client: &ClientSocket, solver: impl BlackBoxFunctionSolver + 'static) -> Self {
        Self { client: client.clone(), root_path: None, solver: WrapperSolver(Box::new(solver)) }
    }
}

pub struct NargoLspService {
    router: Router<LspState>,
}

impl NargoLspService {
    pub fn new(client: &ClientSocket, solver: impl BlackBoxFunctionSolver + 'static) -> Self {
        let state = LspState::new(client, solver);
        let mut router = Router::new(state);
        router
            .request::<request::Initialize, _>(on_initialize)
            .request::<request::Shutdown, _>(on_shutdown)
            .request::<request::CodeLens, _>(on_code_lens_request)
            .request::<request::NargoTests, _>(on_tests_request)
            .request::<request::NargoTestRun, _>(on_test_run_request)
            .notification::<notification::Initialized>(on_initialized)
            .notification::<notification::DidChangeConfiguration>(on_did_change_configuration)
            .notification::<notification::DidOpenTextDocument>(on_did_open_text_document)
            .notification::<notification::DidChangeTextDocument>(on_did_change_text_document)
            .notification::<notification::DidCloseTextDocument>(on_did_close_text_document)
            .notification::<notification::DidSaveTextDocument>(on_did_save_text_document)
            .notification::<notification::Exit>(on_exit);
        Self { router }
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` a normal Service as far as Tower is concerned.
impl Service<AnyRequest> for NargoLspService {
    type Response = JsonValue;
    type Error = ResponseError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.router.poll_ready(cx)
    }

    fn call(&mut self, req: AnyRequest) -> Self::Future {
        self.router.call(req)
    }
}

// This trait implemented as a passthrough to the router, which makes
// our `NargoLspService` able to accept the `async-lsp` middleware.
impl LspService for NargoLspService {
    fn notify(&mut self, notification: AnyNotification) -> ControlFlow<Result<(), Error>> {
        self.router.notify(notification)
    }

    fn emit(&mut self, event: AnyEvent) -> ControlFlow<Result<(), Error>> {
        self.router.emit(event)
    }
}

fn get_package_tests_in_crate(
    context: &Context,
    crate_id: &CrateId,
    crate_name: &CrateName,
) -> Option<Vec<NargoTest>> {
    let fm = &context.file_manager;
    let files = fm.as_file_map();
    let tests =
        context.get_all_test_functions_in_crate_matching(crate_id, FunctionNameMatch::Anything);

    let package_tests: Vec<_> = tests
        .into_iter()
        .map(|(func_name, test_function)| {
            let location = context.function_meta(&test_function.get_id()).name.location;
            let file_id = location.file;

            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();
            let file_uri = Url::from_file_path(fm.path(file_id))
                .expect("Expected a valid file path that can be converted into a URI");

            NargoTest {
                id: NargoTestId::new(crate_name.clone(), func_name.clone()),
                label: func_name,
                uri: file_uri,
                range,
            }
        })
        .collect();

    if package_tests.is_empty() {
        None
    } else {
        Some(package_tests)
    }
}

fn byte_span_to_range<'a, F: files::Files<'a> + ?Sized>(
    files: &'a F,
    file_id: F::FileId,
    span: ops::Range<usize>,
) -> Option<Range> {
    if let Ok(codespan_range) = codespan_lsp::byte_span_to_range(files, file_id, span) {
        // We have to manually construct a Range because the codespan_lsp restricts lsp-types to the wrong version range
        // TODO: codespan is unmaintained and we should probably subsume it. Ref https://github.com/brendanzab/codespan/issues/345
        let range = Range {
            start: Position {
                line: codespan_range.start.line,
                character: codespan_range.start.character,
            },
            end: Position {
                line: codespan_range.end.line,
                character: codespan_range.end.character,
            },
        };
        Some(range)
    } else {
        None
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))] {
        use wasm_bindgen::{prelude::*, JsValue};

        #[wasm_bindgen(module = "@noir-lang/source-resolver")]
        extern "C" {

            #[wasm_bindgen(catch)]
            fn read_file(path: &str) -> Result<String, JsValue>;

        }

        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            let path_str = path_to_file.to_str().unwrap();
            match read_file(path_str) {
                Ok(buffer) => Ok(buffer),
                Err(_) => Err(Error::new(ErrorKind::Other, "could not read file using wasm")),
            }
        }
    } else {
        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(path_to_file)
        }
    }
}
