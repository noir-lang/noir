//! End-to-end test of `nargo lsp` over stdio.
//!
//! The LSP unit tests construct `LspState` directly and call handlers, which bypasses the
//! router, the main-loop `ServerState` and the compiler actor. This test drives the real
//! binary through a realistic session so the message forwarding layer — where ordering and
//! freshness guarantees live — has coverage.
//!
//! Gated to unix because it builds `file://` URIs from paths by hand.
#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use serde_json::{Value, json};

/// Generous timeout per expected message: CI machines can be slow and the first
/// response waits for a full workspace type-check.
const MESSAGE_TIMEOUT: Duration = Duration::from_secs(120);

/// Kills the server on drop so a failing test can't leave an orphan process behind.
struct LspServer {
    child: Child,
    messages: Receiver<Value>,
}

impl Drop for LspServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl LspServer {
    fn start() -> Self {
        let nargo = assert_cmd::cargo::cargo_bin("nargo");
        let mut child = Command::new(nargo)
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to start nargo lsp");

        // Read framed LSP messages on a separate thread so the test can wait with a timeout.
        let stdout = child.stdout.take().unwrap();
        let (sender, messages) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut content_length: Option<usize> = None;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let line = line.trim();
                    if line.is_empty() {
                        break;
                    }
                    if let Some(value) = line.strip_prefix("Content-Length:") {
                        content_length = value.trim().parse().ok();
                    }
                }
                let Some(content_length) = content_length else {
                    return;
                };
                let mut content = vec![0; content_length];
                if reader.read_exact(&mut content).is_err() {
                    return;
                }
                let Ok(message) = serde_json::from_slice::<Value>(&content) else {
                    return;
                };
                if sender.send(message).is_err() {
                    return;
                }
            }
        });

        Self { child, messages }
    }

    fn send(&mut self, message: Value) {
        let content = message.to_string();
        let stdin = self.child.stdin.as_mut().unwrap();
        write!(stdin, "Content-Length: {}\r\n\r\n{}", content.len(), content).unwrap();
        stdin.flush().unwrap();
    }

    /// Reads messages until one has the given request id, skipping notifications the
    /// server sends on its own (diagnostics, test updates, etc.).
    fn wait_for_response(&self, id: i64) -> Value {
        loop {
            let message = self
                .messages
                .recv_timeout(MESSAGE_TIMEOUT)
                .unwrap_or_else(|_| panic!("timed out waiting for response to request {id}"));
            if message.get("id").and_then(Value::as_i64) == Some(id) {
                return message;
            }
        }
    }
}

fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../lsp/test_programs/workspace")
        .canonicalize()
        .expect("could not find the LSP test workspace")
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn document_symbol_names(response: &Value) -> Vec<String> {
    response["result"]
        .as_array()
        .unwrap_or_else(|| panic!("expected documentSymbol array, got: {response}"))
        .iter()
        .map(|symbol| symbol["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn lsp_stdio_session() {
    let workspace = workspace_dir();
    let file_path = workspace.join("two/src/lib.nr");
    let uri = file_uri(&file_path);
    let source = std::fs::read_to_string(&file_path).unwrap();

    let mut server = LspServer::start();

    // Initialize handshake.
    server.send(json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "processId": null, "rootUri": file_uri(&workspace), "capabilities": {} },
    }));
    let response = server.wait_for_response(1);
    assert!(
        response["result"]["capabilities"].is_object(),
        "initialize response should contain capabilities: {response}"
    );
    server.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));

    // Open a document and request its symbols: exercises request forwarding through the
    // compiler actor and back.
    server.send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": {
            "uri": uri, "languageId": "noir", "version": 0, "text": source,
        }},
    }));
    server.send(json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": uri } },
    }));
    let response = server.wait_for_response(2);
    assert_eq!(document_symbol_names(&response), vec!["function_two"]);

    // Two rapid changes followed by requests: the responses must reflect the *latest*
    // text (in-order processing), even though the first change has a syntax error.
    let text_with_fixed = format!("{source}\nfn fixed() {{}}");
    server.send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {
            "textDocument": { "uri": uri, "version": 1 },
            "contentChanges": [{ "text": format!("{source}\nfn broken( {{}}") }],
        },
    }));
    server.send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {
            "textDocument": { "uri": uri, "version": 2 },
            "contentChanges": [{ "text": &text_with_fixed }],
        },
    }));
    // documentSymbol is answered from the main loop's text mirror.
    server.send(json!({
        "jsonrpc": "2.0", "id": 3, "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": uri } },
    }));
    let response = server.wait_for_response(3);
    assert_eq!(document_symbol_names(&response), vec!["function_two", "fixed"]);

    // Hover goes through the compiler actor, so this checks the actor also processed the
    // changes in order: hovering the just-added `fixed` only works on the latest text.
    let fixed_line = text_with_fixed.lines().count() - 1;
    server.send(json!({
        "jsonrpc": "2.0", "id": 6, "method": "textDocument/hover",
        "params": {
            "textDocument": { "uri": uri },
            "position": { "line": fixed_line, "character": 4 },
        },
    }));
    let response = server.wait_for_response(6);
    let hover_text = response["result"].to_string();
    assert!(
        hover_text.contains("fixed"),
        "hover should see the function added by the latest change: {response}"
    );

    // Formatting right after a change (the format-on-save pattern): the returned edits
    // must be based on the newest text.
    server.send(json!({
        "jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {
            "textDocument": { "uri": uri, "version": 3 },
            "contentChanges": [{ "text": format!("{source}\nfn needs_formatting( ) {{}}") }],
        },
    }));
    server.send(json!({
        "jsonrpc": "2.0", "id": 4, "method": "textDocument/formatting",
        "params": {
            "textDocument": { "uri": uri },
            "options": { "tabSize": 4, "insertSpaces": true },
        },
    }));
    let response = server.wait_for_response(4);
    let new_text = response["result"][0]["newText"]
        .as_str()
        .unwrap_or_else(|| panic!("expected formatting edits, got: {response}"));
    assert!(
        new_text.contains("fn needs_formatting() {}"),
        "formatting should reflect the latest change, got: {new_text}"
    );

    // Clean shutdown.
    server.send(json!({ "jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null }));
    server.wait_for_response(5);
    server.send(json!({ "jsonrpc": "2.0", "method": "exit", "params": null }));

    let status = server.child.wait().expect("failed to wait for nargo lsp to exit");
    assert!(status.success(), "nargo lsp should exit cleanly, got: {status}");
}
