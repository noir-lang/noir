use lsp_types::{Position, Range, TextEdit};
use noirc_errors::Location;
use noirc_frontend::{
    ast::{Ident, Path},
    macros_api::ModuleDefId,
};

use crate::{
    byte_span_to_range,
    modules::{get_parent_module_id, module_full_path},
};

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn import_or_qualify(&mut self, path: &Path) {
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
        self.code_actions.push(code_action);
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
        self.code_actions.push(code_action);
    }
}
