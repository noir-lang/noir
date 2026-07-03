#![cfg(test)]

use std::collections::HashMap;
use std::time::Duration;

use acvm::blackbox_solver::StubbedBlackBoxSolver;
use async_lsp::ClientSocket;
use async_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    FormattingOptions, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    Url, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
};

use crate::{CompilerActor, ServerState};

fn new_server_state() -> ServerState {
    let actor = CompilerActor::spawn(ClientSocket::new_closed(), StubbedBlackBoxSolver);
    ServerState { actor, input_files: HashMap::new() }
}

/// Keeps the actor busy until the returned sender is dropped, simulating a long
/// type-check in progress.
fn block_actor(state: &ServerState) -> std::sync::mpsc::Sender<()> {
    let (gate_tx, gate_rx) = std::sync::mpsc::channel::<()>();
    state.actor.notify(move |_state| {
        let _ = gate_rx.recv();
    });
    gate_tx
}

fn open_document(state: &mut ServerState, uri: &Url, text: &str) {
    state.open_document(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "noir".to_string(),
            version: 0,
            text: text.to_string(),
        },
    });
}

fn change_document(state: &mut ServerState, uri: &Url, text: &str) {
    state.change_document(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier { uri: uri.clone(), version: 1 },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }],
    });
}

async fn format_document(state: &ServerState, uri: &Url) -> String {
    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    let future = state.format_document(params);
    // Formatting sits on the save path (format-on-save blocks the editor's save on this
    // reply), so it must be answered promptly even while the actor is busy type-checking.
    let result = tokio::time::timeout(Duration::from_millis(500), future)
        .await
        .expect("formatting must not wait for the compiler actor");
    let edits = result.expect("formatting should succeed").expect("expected formatting edits");
    assert_eq!(edits.len(), 1);
    edits[0].new_text.clone()
}

#[tokio::test]
async fn formatting_is_not_blocked_by_compiler_work() {
    let mut state = new_server_state();
    let _gate = block_actor(&state);

    let uri = Url::parse("file:///main.nr").unwrap();
    open_document(&mut state, &uri, "fn main( ) {}");

    let formatted = format_document(&state, &uri).await;
    assert_eq!(formatted, "fn main() {}\n");
}

#[tokio::test]
async fn formatting_uses_the_latest_document_text() {
    let mut state = new_server_state();
    let _gate = block_actor(&state);

    let uri = Url::parse("file:///main.nr").unwrap();
    open_document(&mut state, &uri, "fn main( ) {}");
    // The actor is blocked, so this change can only be visible to formatting if the main
    // loop keeps its own mirror of document texts.
    change_document(&mut state, &uri, "fn other( ) {}");

    let formatted = format_document(&state, &uri).await;
    assert_eq!(formatted, "fn other() {}\n");
}
