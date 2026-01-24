use std::future::{self, Future};

use async_lsp::{
    ResponseError,
    lsp_types::{Position, TextDocumentPositionParams},
};

use fm::{FileId, FileMap, PathString};
use nargo::{package::Package, workspace::Workspace};
use noirc_driver::CrateId;
use noirc_frontend::{
    hir::{
        FunctionNameMatch,
        def_map::{DefMaps, ModuleId},
        get_all_test_functions_in_crate_matching,
    },
    node_interner::NodeInterner,
};

use crate::{
    LspState, byte_span_to_range,
    requests::process_request,
    types::{CodeLens, CodeLensParams, CodeLensResult, Command},
};

const ARROW: &str = "▶\u{fe0e}";
const GEAR: &str = "⚙";
const TEST_COMMAND: &str = "nargo.test";
const TEST_CODELENS_TITLE: &str = "Run Test";
const COMPILE_COMMAND: &str = "nargo.compile";
const COMPILE_CODELENS_TITLE: &str = "Compile";
const INFO_COMMAND: &str = "nargo.info";
const INFO_CODELENS_TITLE: &str = "Info";
const EXECUTE_COMMAND: &str = "nargo.execute";
const EXECUTE_CODELENS_TITLE: &str = "Execute";
const DEBUG_COMMAND: &str = "nargo.debug.dap";
const DEBUG_CODELENS_TITLE: &str = "Debug";
const DEBUG_TEST_COMMAND: &str = "nargo.debug.test";
const DEBUG_TEST_CODELENS_TITLE: &str = "Debug test";

fn with_arrow(title: &str) -> String {
    format!("{ARROW} {title}")
}

fn package_selection_args(workspace: &Workspace, package: &Package) -> Vec<serde_json::Value> {
    vec![
        "--program-dir".into(),
        workspace.root_dir.display().to_string().into(),
        "--package".into(),
        package.name.to_string().into(),
    ]
}

pub(crate) fn on_code_lens_request(
    state: &mut LspState,
    params: CodeLensParams,
) -> impl Future<Output = Result<CodeLensResult, ResponseError>> + use<> {
    future::ready(on_code_lens_request_inner(state, params))
}

fn on_code_lens_request_inner(
    state: &mut LspState,
    params: CodeLensParams,
) -> Result<CodeLensResult, ResponseError> {
    let Ok(file_path) = params.text_document.uri.to_file_path() else {
        return Ok(None);
    };

    let text_document_position_params = TextDocumentPositionParams {
        text_document: params.text_document,
        position: Position::new(0, 0),
    };
    process_request(state, text_document_position_params, |args| {
        let file_id = args.files.get_file_id(&PathString::from_path(file_path))?;
        let collected_lenses = collect_lenses_for_file(
            file_id,
            args.workspace,
            args.package,
            args.crate_id,
            args.interner,
            args.def_maps,
            args.files,
        );
        if collected_lenses.is_empty() { None } else { Some(collected_lenses) }
    })
}

pub(crate) fn collect_lenses_for_file(
    current_file: FileId,
    workspace: &Workspace,
    package: &Package,
    crate_id: CrateId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    files: &FileMap,
) -> Vec<CodeLens> {
    let mut lenses: Vec<CodeLens> = vec![];

    let tests = get_all_test_functions_in_crate_matching(
        crate_id,
        &FunctionNameMatch::Anything,
        interner,
        def_maps,
    );
    for (func_name, test_function) in tests {
        let location = interner.function_meta(&test_function.id).name.location;
        let file_id = location.file;
        if file_id != current_file {
            continue;
        }

        let range = byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();
        lenses.push(test_lens(workspace, package, &func_name, range));
        lenses.push(debug_test_lens(workspace, package, func_name, range));
    }

    if package.is_binary() {
        if let Some(main_func_id) = def_maps[&crate_id].main_function() {
            let location = interner.function_meta(&main_func_id).name.location;
            let file_id = location.file;
            if file_id == current_file {
                let range =
                    byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();
                lenses.push(compile_lens(workspace, package, range));
                lenses.push(info_lens(workspace, package, range));
                lenses.push(execute_lens(workspace, package, range));
                lenses.push(debug_lens(workspace, package, range));
            }
        }
    }

    if package.is_contract() {
        // Currently not looking to deduplicate this since we don't have a clear decision on if the Contract stuff is staying
        let def_map = def_maps.get(&crate_id).expect("The local crate should be analyzed already");

        for contract in def_map
            .get_all_contracts()
            .map(|(local_id, _)| ModuleId { krate: crate_id, local_id }.module(def_maps))
        {
            let location = contract.location;
            let file_id = location.file;
            if file_id != current_file {
                continue;
            }

            let range =
                byte_span_to_range(files, file_id, location.span.into()).unwrap_or_default();
            lenses.push(compile_lens(workspace, package, range));
            lenses.push(info_lens(workspace, package, range));
        }
    }

    lenses
}

fn info_lens(
    workspace: &Workspace,
    package: &Package,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let info_command = Command {
        title: INFO_CODELENS_TITLE.to_string(),
        command: INFO_COMMAND.into(),
        arguments: Some(package_selection_args(workspace, package)),
    };
    CodeLens { range, command: Some(info_command), data: None }
}

fn execute_lens(
    workspace: &Workspace,
    package: &Package,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let info_command = Command {
        title: EXECUTE_CODELENS_TITLE.to_string(),
        command: EXECUTE_COMMAND.into(),
        arguments: Some(package_selection_args(workspace, package)),
    };
    CodeLens { range, command: Some(info_command), data: None }
}

fn debug_lens(
    workspace: &Workspace,
    package: &Package,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let info_command = Command {
        title: DEBUG_CODELENS_TITLE.to_string(),
        command: DEBUG_COMMAND.into(),
        arguments: Some(package_selection_args(workspace, package)),
    };
    CodeLens { range, command: Some(info_command), data: None }
}

fn compile_lens(
    workspace: &Workspace,
    package: &Package,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let compile_command = Command {
        title: with_arrow(COMPILE_CODELENS_TITLE),
        command: COMPILE_COMMAND.into(),
        arguments: Some(package_selection_args(workspace, package)),
    };
    CodeLens { range, command: Some(compile_command), data: None }
}

fn test_lens(
    workspace: &Workspace,
    package: &Package,
    func_name: &str,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let test_command = Command {
        title: with_arrow(TEST_CODELENS_TITLE),
        command: TEST_COMMAND.into(),
        arguments: Some(
            [
                package_selection_args(workspace, package),
                vec!["--exact".into(), func_name.into(), "--show-output".into()],
            ]
            .concat(),
        ),
    };
    CodeLens { range, command: Some(test_command), data: None }
}

fn debug_test_lens(
    workspace: &Workspace,
    package: &Package,
    func_name: String,
    range: async_lsp::lsp_types::Range,
) -> CodeLens {
    let debug_test_command = Command {
        title: format!("{GEAR} {DEBUG_TEST_CODELENS_TITLE}"),
        command: DEBUG_TEST_COMMAND.into(),
        arguments: Some(
            [package_selection_args(workspace, package), vec!["--exact".into(), func_name.into()]]
                .concat(),
        ),
    };
    CodeLens { range, command: Some(debug_test_command), data: None }
}

#[cfg(test)]
mod tests {

    use async_lsp::lsp_types::{
        CodeLensParams, DidOpenTextDocumentParams, PartialResultParams, TextDocumentIdentifier,
        TextDocumentItem, WorkDoneProgressParams,
    };
    use iter_extended::vecmap;
    use serde_json::Value;
    use tokio::test;

    use crate::{
        notifications::on_did_open_text_document, requests::on_code_lens_request, test_utils,
        types::CodeLensResult,
    };

    async fn get_code_lens(src: &str, directory: &str) -> CodeLensResult {
        let (mut state, noir_text_document) = test_utils::init_lsp_server(directory).await;

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

        on_code_lens_request(
            &mut state,
            CodeLensParams {
                text_document: TextDocumentIdentifier { uri: noir_text_document },
                work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
                partial_result_params: PartialResultParams { partial_result_token: None },
            },
        )
        .await
        .expect("Could not execute on_code_lens_request")
    }

    #[test]
    async fn test_no_code_lens() {
        let src = r#"
        fn foo() {}
        "#;

        let code_lens = get_code_lens(src, "document_symbol").await;
        assert!(code_lens.is_none());
    }

    #[test]
    async fn test_main_code_lens() {
        let src = r#"fn main() {}"#;

        let code_lens = get_code_lens(src, "document_symbol").await.unwrap();
        assert_eq!(code_lens.len(), 4);

        for lens in &code_lens {
            assert_eq!(lens.range.start.line, 0);
            assert_eq!(lens.range.end.line, 0);
            assert_eq!(
                &src[lens.range.start.character as usize..lens.range.end.character as usize],
                "main"
            );
        }

        let mut titles = vecmap(code_lens, |lens| lens.command.unwrap().title);
        titles.sort();

        assert_eq!(titles, vec!["Debug", "Execute", "Info", "▶\u{fe0e} Compile"]);
    }

    #[test]
    async fn test_test_code_lens() {
        let src = r#"mod moo { #[test] fn some_test() {} }"#;

        let code_lens = get_code_lens(src, "document_symbol").await.unwrap();
        assert_eq!(code_lens.len(), 2);

        for lens in &code_lens {
            assert_eq!(lens.range.start.line, 0);
            assert_eq!(lens.range.end.line, 0);
            assert_eq!(
                &src[lens.range.start.character as usize..lens.range.end.character as usize],
                "some_test"
            );
            let arguments = lens.command.as_ref().unwrap().arguments.as_ref().unwrap();
            assert!(arguments.contains(&Value::String("moo::some_test".into())));
        }

        let mut titles = vecmap(code_lens, |lens| lens.command.unwrap().title);
        titles.sort();

        assert_eq!(titles, vec!["▶\u{fe0e} Run Test", "⚙ Debug test"]);
    }

    #[test]
    async fn test_contract_code_lens() {
        let src = r#"contract SomeContract {}"#;

        let code_lens = get_code_lens(src, "test_contract").await.unwrap();
        assert_eq!(code_lens.len(), 2);

        for lens in &code_lens {
            assert_eq!(lens.range.start.line, 0);
            assert_eq!(lens.range.end.line, 0);
            assert_eq!(
                &src[lens.range.start.character as usize..lens.range.end.character as usize],
                "SomeContract"
            );
        }

        let mut titles = vecmap(code_lens, |lens| lens.command.unwrap().title);
        titles.sort();

        assert_eq!(titles, vec!["Info", "▶\u{fe0e} Compile"]);
    }
}
