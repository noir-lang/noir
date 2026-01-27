use std::future::{self, Future};

use async_lsp::{
    ResponseError,
    lsp_types::{Position, TextDocumentPositionParams},
};

use fm::{FileMap, PathString};
use nargo::{package::Package, workspace::Workspace};
use noirc_errors::Span;
use noirc_frontend::{
    ParsedModule,
    ast::{NoirFunction, NoirTrait, NoirTraitImpl, TypeImpl, Visitor},
    graph::CrateGraph,
    hir::def_map::{DefMaps, ModuleId, fully_qualified_module_path},
    node_interner::{NodeInterner, ReferenceId},
    parser::ParsedSubModule,
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
    process_request("code_lens", state, text_document_position_params, |args| {
        let file_id = args.files.get_file_id(&PathString::from_path(file_path))?;
        let file = args.files.get_file(file_id).unwrap();
        let source = file.source();
        let (parsed_module, _errors) = noirc_frontend::parse_program(source, file_id);

        let collected_lenses = collect_lenses_for_file(
            parsed_module,
            args.workspace,
            args.package,
            args.interner,
            args.def_maps,
            args.crate_graph,
            args.files,
        );
        if collected_lenses.is_empty() { None } else { Some(collected_lenses) }
    })
}

pub(crate) fn collect_lenses_for_file(
    parsed_module: ParsedModule,
    workspace: &Workspace,
    package: &Package,
    interner: &NodeInterner,
    def_maps: &DefMaps,
    crate_graph: &CrateGraph,
    files: &FileMap,
) -> Vec<CodeLens> {
    let mut visitor = CodeLensVisitor {
        workspace,
        package,
        interner,
        def_maps,
        crate_graph,
        files,
        nesting: 0,
        lenses: vec![],
    };
    parsed_module.accept(&mut visitor);
    visitor.lenses
}

struct CodeLensVisitor<'a> {
    workspace: &'a Workspace,
    package: &'a Package,
    interner: &'a NodeInterner,
    def_maps: &'a DefMaps,
    crate_graph: &'a CrateGraph,
    files: &'a FileMap,
    nesting: usize,
    lenses: Vec<CodeLens>,
}

impl Visitor for CodeLensVisitor<'_> {
    fn visit_noir_function(&mut self, function: &NoirFunction, _: Span) -> bool {
        let location = function.name_ident().location();

        // Check if it's a main function
        if self.nesting == 0 && self.package.is_binary() && function.name() == "main" {
            let range = byte_span_to_range(self.files, location.file, location.span.into())
                .unwrap_or_default();
            self.lenses.push(compile_lens(self.workspace, self.package, range));
            self.lenses.push(info_lens(self.workspace, self.package, range));
            self.lenses.push(execute_lens(self.workspace, self.package, range));
            self.lenses.push(debug_lens(self.workspace, self.package, range));
        }

        // Check if it's a test function
        if let Some(ReferenceId::Function(func_id)) = self.interner.reference_at_location(location)
        {
            if self.interner.function_modifiers(&func_id).attributes.is_test_function() {
                let func_meta = self.interner.function_meta(&func_id);
                let local_module_id = func_meta.source_module;
                let crate_id = func_meta.source_crate;
                let module_id = ModuleId { krate: crate_id, local_id: local_module_id };
                let module_path = fully_qualified_module_path(
                    self.def_maps,
                    self.crate_graph,
                    &crate_id,
                    module_id,
                );
                let func_name = if module_path.is_empty() {
                    function.name().to_string()
                } else {
                    format!("{}::{}", module_path, function.name())
                };

                let range = byte_span_to_range(self.files, location.file, location.span.into())
                    .unwrap_or_default();
                self.lenses.push(test_lens(self.workspace, self.package, &func_name, range));
                self.lenses.push(debug_test_lens(self.workspace, self.package, func_name, range));
            }
        };

        false
    }

    fn visit_parsed_submodule(&mut self, parsed_sub_module: &ParsedSubModule, _: Span) -> bool {
        // Check if this is a contract
        if parsed_sub_module.is_contract && self.package.is_contract() {
            let location = parsed_sub_module.name.location();
            let range = byte_span_to_range(self.files, location.file, location.span.into())
                .unwrap_or_default();
            self.lenses.push(compile_lens(self.workspace, self.package, range));
            self.lenses.push(info_lens(self.workspace, self.package, range));
        }

        self.nesting += 1;
        parsed_sub_module.accept_children(self);
        self.nesting -= 1;
        false
    }

    fn visit_type_impl(&mut self, _: &TypeImpl, _: Span) -> bool {
        // We don't want to find functions inside impl blocks
        false
    }

    fn visit_noir_trait(&mut self, _: &NoirTrait, _: Span) -> bool {
        // We don't want to find functions inside traits
        false
    }

    fn visit_noir_trait_impl(&mut self, _: &NoirTraitImpl, _: Span) -> bool {
        // We don't want to find functions inside trait impls
        false
    }
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
    async fn test_no_code_lens_on_nested_main() {
        let src = r#"
        mod moo {
            fn main() {}
        }
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
        let src = r#"#[test] fn some_test() {}"#;

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
            assert!(arguments.contains(&Value::String("some_test".into())));
        }

        let mut titles = vecmap(code_lens, |lens| lens.command.unwrap().title);
        titles.sort();

        assert_eq!(titles, vec!["▶\u{fe0e} Run Test", "⚙ Debug test"]);
    }

    #[test]
    async fn test_nested_test_code_lens() {
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
