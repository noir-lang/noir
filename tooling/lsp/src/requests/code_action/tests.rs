use crate::{notifications::on_did_open_text_document, test_utils};

use lsp_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    DidOpenTextDocumentParams, PartialResultParams, Position, Range, TextDocumentIdentifier,
    TextDocumentItem, WorkDoneProgressParams,
};
use tokio::test;

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

#[test]
async fn test_code_action_for_unresolved_path_for_struct() {
    let src = r#"
        mod foo {
            mod bar {
                struct SomeTypeInBar {}
            }
        }

        fn foo(x: SomeType>|<InBar) {}
        "#;

    let actions = get_code_action(src).await;
    assert_eq!(actions.len(), 2);

    let action = &actions[0];
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("Expected an action");
    };

    assert_eq!(action.title, "Import foo::bar::SomeTypeInBar");

    let workspace_edit = action.edit.as_ref().unwrap();
    let text_edits = workspace_edit.changes.as_ref().unwrap().iter().next().unwrap().1;
    assert_eq!(text_edits.len(), 1);

    let text_edit = &text_edits[0];
    assert_eq!(text_edit.new_text, "use foo::bar::SomeTypeInBar;\n");
    assert_eq!(text_edit.range.start.line, 0);
    assert_eq!(text_edit.range.start.character, 0);
    assert_eq!(text_edit.range.end.line, 0);
    assert_eq!(text_edit.range.end.character, 0);

    let action = &actions[1];
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("Expected an action");
    };

    assert_eq!(action.title, "Qualify as foo::bar::SomeTypeInBar");

    let workspace_edit = action.edit.as_ref().unwrap();
    let text_edits = workspace_edit.changes.as_ref().unwrap().iter().next().unwrap().1;
    assert_eq!(text_edits.len(), 1);

    let text_edit = &text_edits[0];
    assert_eq!(text_edit.new_text, "foo::bar::");

    assert_eq!(text_edit.range.start.line, 7);
    assert_eq!(text_edit.range.start.character, 18);
    assert_eq!(text_edit.range.end.line, 7);
    assert_eq!(text_edit.range.end.character, 18);
}

#[test]
async fn test_code_action_for_unresolved_path_for_module() {
    let src = r#"
        mod foo {
            mod bar {
                mod some_module_in_bar {}
            }
        }

        fn main() {
          some_mod>|<ule_in_bar
        }
        "#;

    let actions = get_code_action(src).await;
    assert_eq!(actions.len(), 2);

    let action = &actions[0];
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("Expected an action");
    };

    assert_eq!(action.title, "Import foo::bar::some_module_in_bar");

    let workspace_edit = action.edit.as_ref().unwrap();
    let text_edits = workspace_edit.changes.as_ref().unwrap().iter().next().unwrap().1;
    assert_eq!(text_edits.len(), 1);

    let text_edit = &text_edits[0];
    assert_eq!(text_edit.new_text, "use foo::bar::some_module_in_bar;\n");
    assert_eq!(text_edit.range.start.line, 0);
    assert_eq!(text_edit.range.start.character, 0);
    assert_eq!(text_edit.range.end.line, 0);
    assert_eq!(text_edit.range.end.character, 0);

    let action = &actions[1];
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("Expected an action");
    };

    assert_eq!(action.title, "Qualify as foo::bar::some_module_in_bar");

    let workspace_edit = action.edit.as_ref().unwrap();
    let text_edits = workspace_edit.changes.as_ref().unwrap().iter().next().unwrap().1;
    assert_eq!(text_edits.len(), 1);

    let text_edit = &text_edits[0];
    assert_eq!(text_edit.new_text, "foo::bar::");

    assert_eq!(text_edit.range.start.line, 8);
    assert_eq!(text_edit.range.start.character, 10);
    assert_eq!(text_edit.range.end.line, 8);
    assert_eq!(text_edit.range.end.character, 10);
}
