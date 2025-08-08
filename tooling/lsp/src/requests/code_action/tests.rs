#![cfg(test)]

use crate::{
    notifications::on_did_open_text_document, test_utils, tests::apply_text_edits,
    utils::get_cursor_line_and_column,
};

use async_lsp::lsp_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    DidOpenTextDocumentParams, PartialResultParams, Position, Range, TextDocumentIdentifier,
    TextDocumentItem, WorkDoneProgressParams,
};

use super::on_code_action_request;

/// Given a string with ">|<" (cursor) in it, returns all code actions that are available
/// at that position together with the string with ">|<" removed.
async fn get_code_action(src: &str) -> (CodeActionResponse, String) {
    let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

    let (line, column, src) = get_cursor_line_and_column(src);

    let _ = on_did_open_text_document(
        &mut state,
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: noir_text_document.clone(),
                language_id: "noir".to_string(),
                version: 0,
                text: src.to_string(),
            },
        },
    );

    let position = Position { line: line as u32, character: column as u32 };

    let response = on_code_action_request(
        &mut state,
        CodeActionParams {
            text_document: TextDocumentIdentifier { uri: noir_text_document },
            range: Range { start: position, end: position },
            context: CodeActionContext { diagnostics: Vec::new(), only: None, trigger_kind: None },
            work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
            partial_result_params: PartialResultParams { partial_result_token: None },
        },
    )
    .await
    .expect("Could not execute on_code_action_request")
    .expect("Expected to get a CodeActionResponse, got None");
    (response, src)
}

pub(crate) async fn assert_code_action(title: &str, src: &str, expected: &str) {
    let (actions, src) = get_code_action(src).await;
    let action = actions
        .iter()
        .filter_map(|action| {
            if let CodeActionOrCommand::CodeAction(action) = action {
                if action.title == title { Some(action) } else { None }
            } else {
                None
            }
        })
        .next()
        .expect("Couldn't find an action with the given title");

    let workspace_edit = action.edit.as_ref().unwrap();
    let text_edits = workspace_edit.changes.as_ref().unwrap().iter().next().unwrap().1;

    let result = apply_text_edits(&src.replace(">|<", ""), text_edits);
    if result != expected {
        println!("Expected:\n```\n{expected}\n```\n\nGot:\n```\n{result}\n```");
        assert_eq!(result, expected);
    }
}
