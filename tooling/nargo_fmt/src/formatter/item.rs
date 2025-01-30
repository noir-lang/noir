use noirc_frontend::{
    ast::{ItemVisibility, UseTree},
    hir::resolution::errors::Span,
    parser::{Item, ItemKind},
};

use crate::config::ImportsGranularity;

use super::Formatter;

impl<'a> Formatter<'a> {
    pub(super) fn format_items(&mut self, mut items: Vec<Item>, mut ignore_next: bool) {
        // Reverse the items because we'll be processing them one by one, and it's a bit
        // more efficient to pop than to shift.
        items.reverse();

        while !items.is_empty() {
            // Format the next import group, if there is one.
            let import_group = self.next_import_group(&mut items);
            if let Some(import_group) = import_group {
                self.merge_and_format_imports(import_group.imports, import_group.visibility);
                self.skip_past_span_end_without_formatting(import_group.span_end);
                self.write_line();
                ignore_next = self.ignore_next;

                // Continue from the top because the next thing that comes might be another import group
                continue;
            }

            if let Some(item) = items.pop() {
                self.format_item(item, ignore_next);
                self.write_line();
                ignore_next = self.ignore_next;
            } else {
                break;
            }
        }
    }

    pub(super) fn format_item(&mut self, item: Item, mut ignore_next: bool) {
        self.skip_comments_and_whitespace();

        ignore_next |= self.ignore_next;

        if !item.doc_comments.is_empty() {
            self.format_outer_doc_comments();
            self.skip_comments_and_whitespace();
        }

        ignore_next |= self.ignore_next;

        if ignore_next {
            self.write_and_skip_span_without_formatting(item.span);
            return;
        }

        match item.kind {
            ItemKind::Import(use_tree, item_visibility) => {
                self.format_import(use_tree, item_visibility);
            }
            ItemKind::Function(noir_function) => self.format_function(
                noir_function,
                false, // skip visibility
            ),
            ItemKind::Struct(noir_struct) => self.format_struct(noir_struct),
            ItemKind::Enum(noir_enum) => self.format_enum(noir_enum),
            ItemKind::Trait(noir_trait) => self.format_trait(noir_trait),
            ItemKind::TraitImpl(noir_trait_impl) => self.format_trait_impl(noir_trait_impl),
            ItemKind::Impl(type_impl) => self.format_impl(type_impl),
            ItemKind::TypeAlias(noir_type_alias) => self.format_type_alias(noir_type_alias),
            ItemKind::Global(let_statement, visibility) => {
                self.format_global(let_statement, visibility);
            }
            ItemKind::ModuleDecl(module_declaration) => {
                self.format_module_declaration(module_declaration);
            }
            ItemKind::Submodules(parsed_sub_module) => {
                self.format_submodule(parsed_sub_module);
            }
            ItemKind::InnerAttribute(attribute) => self.format_secondary_attribute(attribute),
        }
    }

    /// Returns the next import group, if there's is one.
    ///
    /// An import group is one or more `use` statements that all have the same visibility,
    /// as long as exactly one newline separates them, and as long as there are no comments
    /// in the `use` statements or trailing comments in them.
    ///
    /// Each import group will be sorted and merged, if the configuration is set to do so.
    fn next_import_group(&self, items: &mut Vec<Item>) -> Option<ImportGroup> {
        if self.config.imports_granularity == ImportsGranularity::Preserve
            && !self.config.reorder_imports
        {
            return None;
        }

        let mut imports = Vec::new();

        let item = items.last()?;
        if self.span_has_comments(item.span) {
            return None;
        }

        let ItemKind::Import(..) = item.kind else {
            return None;
        };

        let item = items.pop().unwrap();
        let ItemKind::Import(use_tree, visibility) = item.kind else {
            panic!("Expected import, got {:?}", item.kind);
        };

        imports.push(use_tree);
        let mut span_end = item.span.end();

        while let Some(item) = items.last() {
            if self.span_is_import_group_separator(Span::from(span_end..item.span.start())) {
                break;
            }

            let next_item_start = if items.len() > 1 {
                if let Some(next_item) = items.get(items.len() - 2) {
                    next_item.span.start()
                } else {
                    self.source.len() as u32
                }
            } else {
                self.source.len() as u32
            };

            if self.span_starts_with_trailing_comment(Span::from(item.span.end()..next_item_start))
            {
                break;
            }

            let ItemKind::Import(_, next_visibility) = &item.kind else {
                break;
            };

            if visibility != *next_visibility {
                break;
            }

            let item = items.pop().unwrap();
            let ItemKind::Import(use_tree, _) = item.kind else {
                panic!("Expected import, got {:?}", item.kind);
            };
            imports.push(use_tree);
            span_end = item.span.end();
        }

        Some(ImportGroup { imports, visibility, span_end })
    }

    fn span_has_comments(&self, span: Span) -> bool {
        let slice = &self.source[span.start() as usize..span.end() as usize];
        slice.contains("/*") || slice.contains("//")
    }

    fn span_starts_with_trailing_comment(&self, span: Span) -> bool {
        let slice = &self.source[span.start() as usize..span.end() as usize];
        slice.trim_start_matches(' ').starts_with("//")
    }

    /// Returns true if there at most one newline in the given span and it contains no comments.
    fn span_is_import_group_separator(&self, span: Span) -> bool {
        let slice = &self.source[span.start() as usize..span.end() as usize];
        let number_of_newlines = slice.chars().filter(|char| *char == '\n').count();
        number_of_newlines > 1 || slice.contains("//") || slice.contains("/*")
    }
}

#[derive(Debug)]
struct ImportGroup {
    imports: Vec<UseTree>,
    visibility: ItemVisibility,
    span_end: u32,
}
