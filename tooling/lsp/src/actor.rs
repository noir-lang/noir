//! The compiler actor owns the [`LspState`] (source overrides, parse caches, type-checked
//! packages) and is the only place compiler work runs. The LSP main loop forwards every
//! request and notification to it as a message and immediately returns to serving protocol
//! traffic, so the editor is never blocked behind a long type-check.
//!
//! Messages are processed strictly in order. This gives the freshness guarantee LSP needs
//! without any bookkeeping: a request enqueued after a document change is processed after
//! the re-check that change triggered, so it always sees up-to-date compiler state.
//!
//! Compiler state cannot cross threads (`NodeInterner` transitively contains `Rc`), which
//! rules out type-checking on a helper thread and handing the result back. Instead the
//! state is created on the thread that processes messages and never leaves it; only
//! messages (jobs and their `Send` results) cross the thread boundary.

use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_lsp::lsp_types::DidChangeTextDocumentParams;
use async_lsp::{ErrorCode, ResponseError};
use tokio::sync::oneshot;

use crate::LspState;
use crate::notifications::on_did_change_text_document;

/// A unit of work for the compiler actor.
pub(crate) enum ActorMessage {
    /// An arbitrary job to run against the compiler state.
    Job(Box<dyn FnOnce(&mut LspState) + Send>),
    /// A document's contents changed. This is a dedicated variant so that consecutive
    /// changes to the same document can be coalesced, type-checking only the latest text.
    FileChanged(DidChangeTextDocumentParams),
}

pub(crate) struct CompilerActor {
    implementation: Implementation,
}

enum Implementation {
    /// Messages run immediately on the caller's thread.
    Inline(Box<LspState>),
}

impl CompilerActor {
    pub(crate) fn inline(state: LspState) -> Self {
        Self { implementation: Implementation::Inline(Box::new(state)) }
    }

    /// Runs a job against the compiler state, resolving with its result once it's processed.
    pub(crate) fn request<T, F>(&mut self, job: F) -> ActorResponse<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut LspState) -> Result<T, ResponseError> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.send(ActorMessage::Job(Box::new(move |state| {
            let _ = tx.send(job(state));
        })));
        ActorResponse(rx)
    }

    /// Enqueues a job that produces no reply.
    pub(crate) fn notify<F>(&mut self, job: F)
    where
        F: FnOnce(&mut LspState) + Send + 'static,
    {
        self.send(ActorMessage::Job(Box::new(job)));
    }

    pub(crate) fn send(&mut self, message: ActorMessage) {
        match &mut self.implementation {
            Implementation::Inline(state) => process_message(state, message),
        }
    }
}

fn process_message(state: &mut LspState, message: ActorMessage) {
    match message {
        ActorMessage::Job(job) => job(state),
        ActorMessage::FileChanged(params) => {
            if let ControlFlow::Break(Err(error)) = on_did_change_text_document(state, params) {
                eprintln!("error processing document change: {error}");
            }
        }
    }
}

/// The pending reply to a job sent with [`CompilerActor::request`].
///
/// Resolves with an error if the actor goes away without replying (for example if the job
/// panicked), so a lost job surfaces as a failed request instead of a hang.
pub(crate) struct ActorResponse<T>(oneshot::Receiver<Result<T, ResponseError>>);

impl<T> Future for ActorResponse<T> {
    type Output = Result<T, ResponseError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(|result| {
            result.unwrap_or_else(|_| {
                Err(ResponseError::new(
                    ErrorCode::INTERNAL_ERROR,
                    "the compiler actor did not reply to the request",
                ))
            })
        })
    }
}
