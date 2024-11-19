use std::{
    collections::{BTreeMap, HashMap},
    future::{self, Future},
    ops::Range,
};

use async_lsp::ResponseError;
use fm::{FileId, FileMap, PathString};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use noirc_errors::Span;
use noirc_frontend::{
    ast::{
        CallExpression, ConstructorExpression, ItemVisibility, MethodCallExpression, NoirTraitImpl,
        Path, UseTree, Visitor,
    },
    graph::CrateId,
    hir::def_map::{CrateDefMap, LocalModuleId, ModuleId},
    node_interner::NodeInterner,
    usage_tracker::UsageTracker,
};
use noirc_frontend::{
    parser::{Item, ItemKind, ParsedSubModule},
    ParsedModule,
};

use crate::{use_segment_positions::UseSegmentPositions, utils, LspState};

use super::{process_request, to_lsp_location};

mod fill_struct_fields;
mod implement_missing_members;
mod import_or_qualify;
mod remove_bang_from_call;
mod remove_unused_import;
mod tests;

pub(crate) fn on_code_action_request(
    state: &mut LspState,
    params: CodeActionParams,
) -> impl Future<Output = Result<Option<CodeActionResponse>, ResponseError>> {
    let uri = params.text_document.clone().uri;
    let position = params.range.start;
    let text_document_position_params =
        TextDocumentPositionParams { text_document: params.text_document, position };

    let result = process_request(state, text_document_position_params, |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        args.files.get_file_id(&path).and_then(|file_id| {
            utils::range_to_byte_span(args.files, file_id, &params.range).and_then(|byte_range| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = CodeActionFinder::new(
                    uri,
                    args.files,
                    file_id,
                    source,
                    byte_range,
                    args.crate_id,
                    args.def_maps,
                    args.interner,
                    args.usage_tracker,
                );
                finder.find(&parsed_module)
            })
        })
    });
    future::ready(result)
}

struct CodeActionFinder<'a> {
    uri: Url,
    files: &'a FileMap,
    file: FileId,
    source: &'a str,
    lines: Vec<&'a str>,
    byte_range: Range<usize>,
    /// The module ID in scope. This might change as we traverse the AST
    /// if we are analyzing something inside an inline module declaration.
    module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    interner: &'a NodeInterner,
    usage_tracker: &'a UsageTracker,
    /// How many nested `mod` we are in deep
    nesting: usize,
    /// The line where an auto_import must be inserted
    auto_import_line: usize,
    use_segment_positions: UseSegmentPositions,
    /// Text edits for the "Remove all unused imports" code action
    unused_imports_text_edits: Vec<TextEdit>,
    code_actions: Vec<CodeAction>,
}

impl<'a> CodeActionFinder<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        uri: Url,
        files: &'a FileMap,
        file: FileId,
        source: &'a str,
        byte_range: Range<usize>,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        interner: &'a NodeInterner,
        usage_tracker: &'a UsageTracker,
    ) -> Self {
        // Find the module the current file belongs to
        let def_map = &def_maps[&krate];
        let local_id = if let Some((module_index, _)) =
            def_map.modules().iter().find(|(_, module_data)| module_data.location.file == file)
        {
            LocalModuleId(module_index)
        } else {
            def_map.root()
        };
        let module_id = ModuleId { krate, local_id };
        Self {
            uri,
            files,
            file,
            source,
            lines: source.lines().collect(),
            byte_range,
            module_id,
            def_maps,
            interner,
            usage_tracker,
            nesting: 0,
            auto_import_line: 0,
            use_segment_positions: UseSegmentPositions::default(),
            unused_imports_text_edits: vec![],
            code_actions: vec![],
        }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CodeActionResponse> {
        parsed_module.accept(self);

        if self.code_actions.is_empty() {
            return None;
        }

        // We also suggest a single "Remove all the unused imports" code action that combines all of the
        // "Remove unused imports" (similar to Rust Analyzer)
        if self.unused_imports_text_edits.len() > 1 {
            let text_edits = std::mem::take(&mut self.unused_imports_text_edits);
            let code_action = self.new_quick_fix_multiple_edits(
                "Remove all the unused imports".to_string(),
                text_edits,
            );
            self.code_actions.push(code_action);
        }

        let mut code_actions = std::mem::take(&mut self.code_actions);
        code_actions.sort_by_key(|code_action| code_action.title.clone());

        Some(code_actions.into_iter().map(CodeActionOrCommand::CodeAction).collect())
    }

    fn new_quick_fix(&self, title: String, text_edit: TextEdit) -> CodeAction {
        self.new_quick_fix_multiple_edits(title, vec![text_edit])
    }

    fn new_quick_fix_multiple_edits(&self, title: String, text_edits: Vec<TextEdit>) -> CodeAction {
        let mut changes = HashMap::new();
        changes.insert(self.uri.clone(), text_edits);

        let workspace_edit = WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };

        CodeAction {
            title,
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(workspace_edit),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        }
    }

    fn includes_span(&self, span: Span) -> bool {
        let byte_range_span = Span::from(self.byte_range.start as u32..self.byte_range.end as u32);
        span.intersects(&byte_range_span)
    }
}

impl<'a> Visitor for CodeActionFinder<'a> {
    fn visit_item(&mut self, item: &Item) -> bool {
        if let ItemKind::Import(use_tree, _) = &item.kind {
            if let Some(lsp_location) = to_lsp_location(self.files, self.file, item.span) {
                self.auto_import_line = (lsp_location.range.end.line + 1) as usize;
            }
            self.use_segment_positions.add(use_tree);
        }

        self.includes_span(item.span)
    }

    fn visit_parsed_submodule(&mut self, parsed_sub_module: &ParsedSubModule, span: Span) -> bool {
        // Switch `self.module_id` to the submodule
        let previous_module_id = self.module_id;

        let def_map = &self.def_maps[&self.module_id.krate];
        let Some(module_data) = def_map.modules().get(self.module_id.local_id.0) else {
            return false;
        };
        if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
            self.module_id = ModuleId { krate: self.module_id.krate, local_id: *child_module };
        }

        let old_auto_import_line = self.auto_import_line;
        self.nesting += 1;

        if let Some(lsp_location) = to_lsp_location(self.files, self.file, span) {
            self.auto_import_line = (lsp_location.range.start.line + 1) as usize;
        }

        parsed_sub_module.contents.accept(self);

        // Restore the old module before continuing
        self.module_id = previous_module_id;
        self.nesting -= 1;
        self.auto_import_line = old_auto_import_line;

        false
    }

    fn visit_import(&mut self, use_tree: &UseTree, span: Span, visibility: ItemVisibility) -> bool {
        self.remove_unused_import(use_tree, visibility, span);

        true
    }

    fn visit_path(&mut self, path: &Path) {
        self.import_or_qualify(path);
    }

    fn visit_constructor_expression(
        &mut self,
        constructor: &ConstructorExpression,
        span: Span,
    ) -> bool {
        self.fill_struct_fields(constructor, span);

        true
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, span: Span) -> bool {
        self.implement_missing_members(noir_trait_impl, span);

        true
    }

    fn visit_call_expression(&mut self, call: &CallExpression, span: Span) -> bool {
        if !self.includes_span(span) {
            return false;
        }

        if call.is_macro_call {
            self.remove_bang_from_call(call.func.span);
        }

        true
    }

    fn visit_method_call_expression(
        &mut self,
        method_call: &MethodCallExpression,
        span: Span,
    ) -> bool {
        if !self.includes_span(span) {
            return false;
        }

        if method_call.is_macro_call {
            self.remove_bang_from_call(method_call.method_name.span());
        }

        true
    }
}
