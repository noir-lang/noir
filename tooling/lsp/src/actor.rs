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
use std::sync::mpsc;
use std::task::{Context, Poll};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use async_lsp::lsp_types::DidChangeTextDocumentParams;
use async_lsp::{ClientSocket, ErrorCode, ResponseError};
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
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Inline(Box<LspState>),
    /// Messages are processed in order by a dedicated thread that owns the compiler state.
    Threaded(mpsc::Sender<ActorMessage>),
}

impl CompilerActor {
    /// Creates an actor that processes messages immediately on the caller's thread.
    ///
    /// This is the fallback for targets without threads (wasm); it keeps the LSP correct
    /// there at the cost of blocking the main loop during compiler work.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) fn inline(state: LspState) -> Self {
        Self { implementation: Implementation::Inline(Box::new(state)) }
    }

    /// Spawns a dedicated thread owning the compiler state and processing messages in order.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn spawn(
        client: ClientSocket,
        solver: impl BlackBoxFunctionSolver<FieldElement> + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("noir-compiler".to_string())
            .spawn(move || {
                let mut state = LspState::new(&client, solver);
                run_actor_loop(&mut state, receiver);
            })
            .expect("failed to spawn the compiler actor thread");
        Self { implementation: Implementation::Threaded(sender) }
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
            Implementation::Threaded(sender) => {
                // If the actor thread is gone the message's reply channel (if any) is
                // dropped with the message, so the corresponding request resolves with an
                // error instead of hanging.
                if sender.send(message).is_err() {
                    eprintln!("the compiler actor thread is gone; dropping message");
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_actor_loop(state: &mut LspState, receiver: mpsc::Receiver<ActorMessage>) {
    while let Ok(message) = receiver.recv() {
        // Drain whatever queued up while the previous message was being processed, so
        // bursts of document changes (fast typing during a slow re-check) can be coalesced
        // instead of type-checked one by one.
        let mut batch = vec![message];
        while let Ok(message) = receiver.try_recv() {
            batch.push(message);
        }

        for message in coalesce_file_changes(batch) {
            // A panicking job (e.g. an ICE during type-checking) must not take this thread
            // down with it: that would leave every queued and future request unanswered.
            // The unwind drops the job's reply channel, so its request resolves with an
            // error, and the loop continues with the next message. Caches the panic left
            // incomplete are rebuilt from sources the next time they are found missing.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                process_message(state, message);
            }));
            if result.is_err() {
                eprintln!("a compiler actor job panicked; continuing with the next message");
            }
        }
    }
}

/// Merges every run of consecutive `FileChanged` messages for the same document into the
/// last change of the run, which carries the newest text (the server uses full-document
/// sync). Only adjacent messages are merged, so a message ordered between two changes
/// (e.g. a request) still observes the document text of the change that preceded it.
fn coalesce_file_changes(batch: Vec<ActorMessage>) -> Vec<ActorMessage> {
    let mut messages: Vec<ActorMessage> = Vec::with_capacity(batch.len());
    for message in batch {
        match (messages.last_mut(), message) {
            (Some(ActorMessage::FileChanged(previous)), ActorMessage::FileChanged(next))
                if previous.text_document.uri == next.text_document.uri =>
            {
                *previous = next;
            }
            (_, message) => messages.push(message),
        }
    }
    messages
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

#[cfg(test)]
mod tests {
    use super::*;
    use acvm::blackbox_solver::StubbedBlackBoxSolver;
    use async_lsp::lsp_types::{
        TextDocumentContentChangeEvent, Url, VersionedTextDocumentIdentifier,
    };

    fn spawn_actor() -> CompilerActor {
        CompilerActor::spawn(ClientSocket::new_closed(), StubbedBlackBoxSolver)
    }

    #[tokio::test]
    async fn processes_messages_in_order() {
        let mut actor = spawn_actor();

        // A slow fire-and-forget job followed by a request: the request must observe the
        // job's effect even though the job is still running when the request is enqueued.
        actor.notify(|state| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            state.options.enable_code_lens = false;
        });
        let enabled = actor.request(|state| Ok(state.options.enable_code_lens)).await.unwrap();
        assert!(!enabled);
    }

    #[tokio::test]
    async fn survives_a_panicking_job() {
        let mut actor = spawn_actor();

        let result =
            actor.request(|_state| -> Result<i32, ResponseError> { panic!("job panicked") }).await;
        assert!(result.is_err(), "a panicked request should resolve with an error, not hang");

        let answer = actor.request(|_state| Ok(42)).await;
        assert_eq!(answer.unwrap(), 42, "the actor should keep serving requests after a panic");
    }

    fn file_changed(uri: &Url, text: &str) -> ActorMessage {
        ActorMessage::FileChanged(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri: uri.clone(), version: 0 },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        })
    }

    /// Renders a message as `uri=text` for `FileChanged` and `job` for `Job`, so tests can
    /// assert on whole batches at a glance.
    fn describe(message: &ActorMessage) -> String {
        match message {
            ActorMessage::Job(_) => "job".to_string(),
            ActorMessage::FileChanged(params) => {
                format!("{}={}", params.text_document.uri, params.content_changes[0].text)
            }
        }
    }

    #[test]
    fn coalesces_consecutive_changes_to_the_same_document() {
        let one = Url::parse("file:///one.nr").unwrap();
        let two = Url::parse("file:///two.nr").unwrap();

        let batch = vec![
            file_changed(&one, "a"),
            file_changed(&one, "b"),
            file_changed(&two, "c"),
            file_changed(&one, "d"),
        ];
        let messages = coalesce_file_changes(batch);
        let descriptions: Vec<_> = messages.iter().map(describe).collect();
        assert_eq!(descriptions, vec!["file:///one.nr=b", "file:///two.nr=c", "file:///one.nr=d"]);
    }

    #[test]
    fn does_not_coalesce_changes_across_other_messages() {
        let one = Url::parse("file:///one.nr").unwrap();

        let batch = vec![
            file_changed(&one, "a"),
            ActorMessage::Job(Box::new(|_state| ())),
            file_changed(&one, "b"),
        ];
        let messages = coalesce_file_changes(batch);
        let descriptions: Vec<_> = messages.iter().map(describe).collect();
        assert_eq!(descriptions, vec!["file:///one.nr=a", "job", "file:///one.nr=b"]);
    }
}
