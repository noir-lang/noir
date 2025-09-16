use async_lsp::lsp_types::{self, FoldingRange, FoldingRangeKind};
use fm::{FileId, FileMap};
use noirc_errors::Span;
use noirc_frontend::ast::{ItemVisibility, ModuleDeclaration, UseTree, Visitor};

use crate::requests::to_lsp_location;

pub(super) struct NodesCollector<'files> {
    file_id: FileId,
    files: &'files FileMap,
    ranges: Vec<FoldingRange>,
    module_group: Option<Group>,
    use_group: Option<Group>,
}

struct Group {
    start: lsp_types::Location,
    end: lsp_types::Location,
    count: usize,
}

impl<'files> NodesCollector<'files> {
    pub(super) fn new(file_id: FileId, files: &'files FileMap) -> Self {
        Self { file_id, files, ranges: Vec::new(), module_group: None, use_group: None }
    }

    pub(super) fn collect(mut self, source: &str) -> Vec<FoldingRange> {
        let (parsed_module, _errors) = noirc_frontend::parse_program(source, self.file_id);
        parsed_module.accept(&mut self);

        if let Some(group) = &self.module_group {
            Self::push_group(group, None, &mut self.ranges);
        }

        if let Some(group) = &self.use_group {
            Self::push_group(group, Some(FoldingRangeKind::Imports), &mut self.ranges);
        }

        self.ranges
    }

    fn push_group(group: &Group, kind: Option<FoldingRangeKind>, ranges: &mut Vec<FoldingRange>) {
        if group.count == 1 {
            return;
        }

        let start_line = group.start.range.start.line;
        let end_line = group.end.range.end.line;
        if start_line == end_line {
            return;
        }

        ranges.push(FoldingRange {
            start_line,
            start_character: None,
            end_line,
            end_character: None,
            kind,
            collapsed_text: None,
        });
    }
}

impl Visitor for NodesCollector<'_> {
    fn visit_module_declaration(&mut self, _: &ModuleDeclaration, span: Span) {
        let Some(location) = to_lsp_location(self.files, self.file_id, span) else {
            return;
        };

        if let Some(group) = &mut self.module_group {
            if location.range.start.line - group.end.range.end.line <= 1 {
                group.end = location;
                group.count += 1;
                return;
            }

            Self::push_group(group, None, &mut self.ranges);
        }

        self.module_group = Some(Group { start: location.clone(), end: location, count: 1 });
    }

    fn visit_import(&mut self, _: &UseTree, span: Span, _visibility: ItemVisibility) -> bool {
        let Some(location) = to_lsp_location(self.files, self.file_id, span) else {
            return true;
        };

        if let Some(group) = &mut self.use_group {
            if location.range.start.line - group.end.range.end.line <= 1 {
                group.end = location;
                group.count += 1;
                return true;
            }

            Self::push_group(group, Some(FoldingRangeKind::Imports), &mut self.ranges);
        }

        self.use_group = Some(Group { start: location.clone(), end: location, count: 1 });

        true
    }

    fn visit_use_tree(&mut self, use_tree: &UseTree) -> bool {
        let Some(location) = to_lsp_location(self.files, self.file_id, use_tree.location.span)
        else {
            return true;
        };

        // If the use tree spans multiple lines, we can fold it
        if location.range.start.line != location.range.end.line {
            self.ranges.push(FoldingRange {
                start_line: location.range.start.line,
                start_character: None,
                end_line: location.range.end.line,
                end_character: None,
                kind: Some(FoldingRangeKind::Imports),
                collapsed_text: None,
            });
        }

        true
    }
}
