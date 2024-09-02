use std::{
    collections::{BTreeMap, HashMap},
    future::{self, Future},
};

use async_lsp::ResponseError;
use fm::{FileId, FileMap, PathString};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    Position, Range, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{ConstructorExpression, Ident, Path, Visitor},
    graph::CrateId,
    hir::def_map::{CrateDefMap, LocalModuleId, ModuleId},
    macros_api::{ModuleDefId, NodeInterner},
    node_interner::ReferenceId,
    parser::{Item, ItemKind, ParsedSubModule},
    ParsedModule,
};

use crate::{
    byte_span_to_range,
    modules::{get_parent_module_id, module_full_path},
    utils, LspState,
};

use super::{process_request, to_lsp_location};

#[cfg(test)]
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
            utils::position_to_byte_index(args.files, file_id, &position).and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = CodeActionFinder::new(
                    uri,
                    args.files,
                    file_id,
                    source,
                    byte_index,
                    args.crate_id,
                    args.def_maps,
                    args.interner,
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
    byte_index: usize,
    /// The module ID in scope. This might change as we traverse the AST
    /// if we are analyzing something inside an inline module declaration.
    module_id: ModuleId,
    def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
    interner: &'a NodeInterner,
    /// How many nested `mod` we are in deep
    nesting: usize,
    /// The line where an auto_import must be inserted
    auto_import_line: usize,
    code_actions: Vec<CodeActionOrCommand>,
}

impl<'a> CodeActionFinder<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        uri: Url,
        files: &'a FileMap,
        file: FileId,
        source: &'a str,
        byte_index: usize,
        krate: CrateId,
        def_maps: &'a BTreeMap<CrateId, CrateDefMap>,
        interner: &'a NodeInterner,
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
            byte_index,
            module_id,
            def_maps,
            interner,
            nesting: 0,
            auto_import_line: 0,
            code_actions: vec![],
        }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<CodeActionResponse> {
        parsed_module.accept(self);

        if self.code_actions.is_empty() {
            return None;
        }

        let mut code_actions = std::mem::take(&mut self.code_actions);
        code_actions.sort_by_key(|code_action| {
            let CodeActionOrCommand::CodeAction(code_action) = code_action else {
                panic!("We only gather code actions, never commands");
            };
            code_action.title.clone()
        });

        Some(code_actions)
    }

    fn push_import_code_action(&mut self, full_path: &str) {
        let line = self.auto_import_line as u32;
        let character = (self.nesting * 4) as u32;
        let indent = " ".repeat(self.nesting * 4);
        let mut newlines = "\n";

        // If the line we are inserting into is not an empty line, insert an extra line to make some room
        if let Some(line_text) = self.lines.get(line as usize) {
            if !line_text.trim().is_empty() {
                newlines = "\n\n";
            }
        }

        let title = format!("Import {}", full_path);
        let text_edit = TextEdit {
            range: Range { start: Position { line, character }, end: Position { line, character } },
            new_text: format!("use {};{}{}", full_path, newlines, indent),
        };

        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(CodeActionOrCommand::CodeAction(code_action));
    }

    fn push_qualify_code_action(&mut self, ident: &Ident, prefix: &str, full_path: &str) {
        let Some(range) = byte_span_to_range(
            self.files,
            self.file,
            ident.span().start() as usize..ident.span().start() as usize,
        ) else {
            return;
        };

        let title = format!("Qualify as {}", full_path);
        let text_edit = TextEdit { range, new_text: format!("{}::", prefix) };

        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(CodeActionOrCommand::CodeAction(code_action));
    }

    fn new_quick_fix(&self, title: String, text_edit: TextEdit) -> CodeAction {
        let mut changes = HashMap::new();
        changes.insert(self.uri.clone(), vec![text_edit]);

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
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl<'a> Visitor for CodeActionFinder<'a> {
    fn visit_item(&mut self, item: &Item) -> bool {
        if let ItemKind::Import(..) = &item.kind {
            if let Some(lsp_location) = to_lsp_location(self.files, self.file, item.span) {
                self.auto_import_line = (lsp_location.range.end.line + 1) as usize;
            }
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

    fn visit_path(&mut self, path: &Path) {
        if path.segments.len() != 1 {
            return;
        }

        let ident = &path.segments[0].ident;
        if !self.includes_span(ident.span()) {
            return;
        }

        let location = Location::new(ident.span(), self.file);
        if self.interner.find_referenced(location).is_some() {
            return;
        }

        let current_module_parent_id = get_parent_module_id(self.def_maps, self.module_id);

        // The Path doesn't resolve to anything so it means it's an error and maybe we
        // can suggest an import or to fully-qualify the path.
        for (name, entries) in self.interner.get_auto_import_names() {
            if name != &ident.0.contents {
                continue;
            }

            for (module_def_id, visibility) in entries {
                let Some(module_full_path) = module_full_path(
                    *module_def_id,
                    *visibility,
                    self.module_id,
                    current_module_parent_id,
                    self.interner,
                ) else {
                    continue;
                };

                let full_path = if let ModuleDefId::ModuleId(..) = module_def_id {
                    module_full_path.clone()
                } else {
                    format!("{}::{}", module_full_path, name)
                };

                let qualify_prefix = if let ModuleDefId::ModuleId(..) = module_def_id {
                    let mut segments: Vec<_> = module_full_path.split("::").collect();
                    segments.pop();
                    segments.join("::")
                } else {
                    module_full_path
                };

                self.push_import_code_action(&full_path);
                self.push_qualify_code_action(ident, &qualify_prefix, &full_path);
            }
        }
    }

    fn visit_constructor_expression(
        &mut self,
        constructor: &ConstructorExpression,
        span: Span,
    ) -> bool {
        if !self.includes_span(span) {
            return false;
        }

        // Find out which struct this is
        let location = Location::new(constructor.type_name.last_ident().span(), self.file);
        let Some(ReferenceId::Struct(struct_id)) = self.interner.find_referenced(location) else {
            return true;
        };

        let struct_type = self.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();

        // First get all of the struct's fields
        let mut fields = struct_type.get_fields_as_written();

        // Remove the ones that already exists in the constructor
        for (field, _) in &constructor.fields {
            fields.retain(|(name, _)| name != &field.0.contents);
        }

        if fields.is_empty() {
            return true;
        }

        // Some fields are missing. Let's suggest a quick fix that adds them.
        let bytes = self.source.as_bytes();
        let right_brace_index = span.end() as usize - 1;
        let mut index = right_brace_index - 1;
        while bytes[index].is_ascii_whitespace() {
            index -= 1;
        }

        let char_before_right_brace = bytes[index] as char;

        index += 1;

        let Some(range) = byte_span_to_range(self.files, self.file, index..index) else {
            return true;
        };

        // If the constructor spans multiple lines, we'll add the new fields in new lines too.
        // Otherwise we'll add all the fields in a single line.
        let constructor_range =
            byte_span_to_range(self.files, self.file, span.start() as usize..span.end() as usize);

        // If it's multiline, find out the indent of the beginning line: we'll add new fields
        // with that indent "plus one" (4 more spaces).
        let line_indent = if let Some(constructor_range) = constructor_range {
            if constructor_range.start.line == constructor_range.end.line {
                None
            } else {
                let line = self.lines[constructor_range.start.line as usize];
                let whitespace_bytes =
                    line.bytes().take_while(|byte| byte.is_ascii_whitespace()).count();
                Some(whitespace_bytes)
            }
        } else {
            None
        };
        let line_indent = line_indent.map(|indent| " ".repeat(indent + 4));

        let on_whitespace = bytes[index].is_ascii_whitespace();

        let mut new_text = String::new();

        // Add a comma if there's not a trailing one (if there are existing fields)
        if !constructor.fields.is_empty() && char_before_right_brace != ',' {
            new_text.push(',');
        }

        // Add space or newline depending on whether it's multiline or not
        if let Some(line_indent) = &line_indent {
            new_text.push('\n');
            new_text.push_str(line_indent);
        } else if !on_whitespace || constructor.fields.is_empty() {
            new_text.push(' ');
        }

        for (index, (name, _)) in fields.iter().enumerate() {
            if index > 0 {
                new_text.push(',');
                if let Some(line_indent) = &line_indent {
                    new_text.push('\n');
                    new_text.push_str(line_indent);
                } else {
                    new_text.push(' ');
                }
            }
            new_text.push_str(name);
            new_text.push_str(": ()");
        }

        if !bytes[right_brace_index - 1].is_ascii_whitespace() {
            new_text.push(' ');
        }

        let title = "Fill struct fields".to_string();
        let text_edit = TextEdit { range, new_text };
        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(CodeActionOrCommand::CodeAction(code_action));

        true
    }
}
