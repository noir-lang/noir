use lsp_types::{Position, Range, TextEdit};

use noirc_errors::Span;
use noirc_frontend::hir::def_map::ModuleDefId;

use crate::{
    modules::{relative_module_full_path, relative_module_id_path},
    requests::to_lsp_location,
};

use super::{
    kinds::{FunctionCompletionKind, FunctionKind, RequestedItems},
    name_matches,
    sort_text::auto_import_sort_text,
    NodeFinder, UseSegmentPosition,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn complete_auto_imports(
        &mut self,
        prefix: &str,
        requested_items: RequestedItems,
        function_completion_kind: FunctionCompletionKind,
    ) {
        let current_module_parent_id = self.module_id.parent(self.def_maps);

        for (name, entries) in self.interner.get_auto_import_names() {
            if !name_matches(name, prefix) {
                continue;
            }

            for (module_def_id, visibility, defining_module) in entries {
                if self.suggested_module_def_ids.contains(module_def_id) {
                    continue;
                }

                let completion_items = self.module_def_id_completion_items(
                    *module_def_id,
                    name.clone(),
                    function_completion_kind,
                    FunctionKind::Any,
                    requested_items,
                );

                if completion_items.is_empty() {
                    continue;
                };

                self.suggested_module_def_ids.insert(*module_def_id);

                for mut completion_item in completion_items {
                    let module_full_path = if let Some(defining_module) = defining_module {
                        relative_module_id_path(
                            *defining_module,
                            &self.module_id,
                            current_module_parent_id,
                            self.interner,
                        )
                    } else {
                        let Some(module_full_path) = relative_module_full_path(
                            *module_def_id,
                            *visibility,
                            self.module_id,
                            current_module_parent_id,
                            self.interner,
                            self.def_maps,
                        ) else {
                            continue;
                        };
                        module_full_path
                    };

                    let full_path = if defining_module.is_some()
                        || !matches!(module_def_id, ModuleDefId::ModuleId(..))
                    {
                        format!("{}::{}", module_full_path, name)
                    } else {
                        module_full_path
                    };

                    let mut label_details = completion_item.label_details.unwrap();
                    label_details.detail = Some(format!("(use {})", full_path));
                    completion_item.label_details = Some(label_details);

                    // See if there's a single place where one of the parent paths is located
                    let (use_segment_position, name) =
                        self.find_use_segment_position(full_path.clone());
                    match use_segment_position {
                        UseSegmentPosition::NoneOrMultiple => {
                            // The parent path either isn't in any use statement, or it exists in multiple
                            // use statements. In either case we'll add a new use statement.
                            completion_item.additional_text_edits =
                                Some(self.new_use_completion_item_additional_text_edits(full_path));
                        }
                        UseSegmentPosition::Last { span } => {
                            // We have
                            //
                            // use foo::bar;
                            //          ^^^ -> span
                            //
                            // and we want to transform it to:
                            //
                            // use foo::bar::{self, baz};
                            //             ^^^^^^^^^^^^^
                            //
                            // So we need one text edit:
                            // 1. insert "::{self, baz}" right after the span
                            if let Some(lsp_location) = to_lsp_location(self.files, self.file, span)
                            {
                                let range = lsp_location.range;
                                completion_item.additional_text_edits = Some(vec![TextEdit {
                                    new_text: format!("::{{self, {}}}", name),
                                    range: Range { start: range.end, end: range.end },
                                }]);
                            } else {
                                completion_item.additional_text_edits = Some(
                                    self.new_use_completion_item_additional_text_edits(full_path),
                                );
                            }
                        }
                        UseSegmentPosition::BeforeSegment { segment_span_until_end } => {
                            // Go past the end
                            let segment_span_until_end = Span::from(
                                segment_span_until_end.start()..segment_span_until_end.end() + 1,
                            );

                            // We have
                            //
                            // use foo::bar::{one, two};
                            //          ^^^^^^^^^^^^^^^ -> segment_span_until_end
                            //
                            // and we want to transform it to:
                            //
                            // use foo::{bar::{one, two}, baz};
                            //          ^               ^^^^^^
                            //
                            // So we need two text edits:
                            // 1. insert "{" right before the segment span
                            // 2. insert ", baz}" right after the segment span
                            if let Some(lsp_location) =
                                to_lsp_location(self.files, self.file, segment_span_until_end)
                            {
                                let range = lsp_location.range;
                                completion_item.additional_text_edits = Some(vec![
                                    TextEdit {
                                        new_text: "{".to_string(),
                                        range: Range { start: range.start, end: range.start },
                                    },
                                    TextEdit {
                                        new_text: format!(", {}}}", name),
                                        range: Range { start: range.end, end: range.end },
                                    },
                                ]);
                            } else {
                                completion_item.additional_text_edits = Some(
                                    self.new_use_completion_item_additional_text_edits(full_path),
                                );
                            }
                        }
                        UseSegmentPosition::BeforeList { first_entry_span, list_is_empty } => {
                            // We have
                            //
                            // use foo::bar::{one, two};
                            //                ^^^ -> first_entry_span
                            //
                            // and we want to transform it to:
                            //
                            // use foo::bar::{baz, one, two};
                            //                ^^^^
                            //
                            // So we need one text edit:
                            // 1. insert "baz, " right before the first entry span
                            if let Some(lsp_location) =
                                to_lsp_location(self.files, self.file, first_entry_span)
                            {
                                let range = lsp_location.range;
                                completion_item.additional_text_edits = Some(vec![TextEdit {
                                    new_text: if list_is_empty {
                                        name
                                    } else {
                                        format!("{}, ", name)
                                    },
                                    range: Range { start: range.start, end: range.start },
                                }]);
                            } else {
                                completion_item.additional_text_edits = Some(
                                    self.new_use_completion_item_additional_text_edits(full_path),
                                );
                            }
                        }
                    }

                    completion_item.sort_text = Some(auto_import_sort_text());

                    self.completion_items.push(completion_item);
                }
            }
        }
    }

    fn new_use_completion_item_additional_text_edits(&self, full_path: String) -> Vec<TextEdit> {
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

        vec![TextEdit {
            range: Range { start: Position { line, character }, end: Position { line, character } },
            new_text: format!("use {};{}{}", full_path, newlines, indent),
        }]
    }

    /// Given a full path like `foo::bar::baz`, returns the first non-"NoneOrMultiple" segment position
    /// trying each successive parent, together with the name after the parent.
    ///
    /// For example, first we'll check if `foo::bar` has a single position. If not, we'll try with `foo`.
    fn find_use_segment_position(&self, full_path: String) -> (UseSegmentPosition, String) {
        // Build a parent path to know in which full segment we need to add this import
        let mut segments: Vec<_> = full_path.split("::").collect();
        let mut name = segments.pop().unwrap().to_string();
        let mut parent_path = segments.join("::");

        loop {
            let use_segment_position =
                self.use_segment_positions.get(&parent_path).cloned().unwrap_or_default();

            if let UseSegmentPosition::NoneOrMultiple = use_segment_position {
                if let Some(next_name) = segments.pop() {
                    name = format!("{next_name}::{name}");
                    parent_path = segments.join("::");
                } else {
                    return (UseSegmentPosition::NoneOrMultiple, String::new());
                }
            } else {
                return (use_segment_position, name);
            }
        }
    }
}
