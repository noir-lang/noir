#![cfg(test)]

use crate::{notifications::on_did_open_text_document, test_utils, tests::apply_text_edits};

use lsp_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    DidOpenTextDocumentParams, PartialResultParams, Position, Range, TextDocumentIdentifier,
    TextDocumentItem, WorkDoneProgressParams,
};

use super::on_code_action_request;

async fn get_code_action(src: &str) -> CodeActionResponse {
    let (mut state, noir_text_document) = test_utils::init_lsp_server("document_symbol").await;

    let (line, column) = src
        .lines()
        .enumerate()
        .find_map(|(line_index, line)| line.find(">|<").map(|char_index| (line_index, char_index)))
        .expect("Expected to find one >|< in the source code");

    let src = src.replace(">|<", "");

    on_did_open_text_document(
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

    on_code_action_request(
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
    .unwrap()
}

pub(crate) async fn assert_code_action(title: &str, src: &str, expected: &str) {
    let actions = get_code_action(src).await;
    let action = actions
        .iter()
        .filter_map(|action| {
            if let CodeActionOrCommand::CodeAction(action) = action {
                if action.title == title {
                    Some(action)
                } else {
                    None
                }
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
        println!("Expected:\n```\n{}\n```\n\nGot:\n```\n{}\n```", expected, result);
        assert_eq!(result, expected);
    }
}
