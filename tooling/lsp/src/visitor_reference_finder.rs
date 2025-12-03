use async_lsp::lsp_types;
use fm::FileId;
use nargo_doc::links::{LinkFinder, LinkTarget};
use noirc_errors::Span;
use noirc_frontend::{
    ParsedModule,
    ast::{
        AttributeTarget, LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait,
        TypeAlias, Visitor,
    },
    hir::{
        def_map::{LocalModuleId, ModuleId},
        resolution::import::resolve_import,
    },
    modules::module_def_id_to_reference_id,
    node_interner::ReferenceId,
    parser::ParsedSubModule,
    token::{MetaAttribute, MetaAttributeName},
    usage_tracker::UsageTracker,
};

use crate::{
    doc_comments::current_module_and_type,
    requests::{ProcessRequestCallbackArgs, to_lsp_location},
};

/// Traverses an AST to find a ReferenceId at the given byte index.
/// This searches:
/// - references in doc comment links
/// - attribute references (see `visit_meta_attribute`).
pub(crate) struct VisitorReferenceFinder<'a> {
    source: &'a str,
    byte_index: usize,
    /// The module ID in scope. This might change as we traverse the AST
    /// if we are analyzing something inside an inline module declaration.
    module_id: ModuleId,
    args: &'a ProcessRequestCallbackArgs<'a>,
    link_finder: LinkFinder,

    /// The found ReferenceId, if any, along with an LSP location that covers
    /// the range of the text that points to that reference (None if the range
    /// should be the word under the cursor, or Some if the range is not that word,
    /// which is the case of doc comment links where the entire `[...]` is the range)
    reference_id: Option<(ReferenceId, Option<lsp_types::Location>)>,
}

impl<'a> VisitorReferenceFinder<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        file: FileId,
        source: &'a str,
        byte_index: usize,
        args: &'a ProcessRequestCallbackArgs<'a>,
    ) -> Self {
        // Find the module the current file belongs to
        let krate = args.crate_id;
        let def_map = &args.def_maps[&krate];
        let local_id = if let Some((module_index, _)) =
            def_map.modules().iter().find(|(_, module_data)| module_data.location.file == file)
        {
            LocalModuleId::new(module_index)
        } else {
            def_map.root()
        };
        let module_id = ModuleId { krate, local_id };
        let link_finder = LinkFinder::default();
        Self { source, byte_index, module_id, args, link_finder, reference_id: None }
    }

    pub(crate) fn find(
        &mut self,
        parsed_module: &ParsedModule,
    ) -> Option<(ReferenceId, Option<lsp_types::Location>)> {
        parsed_module.accept(self);

        std::mem::take(&mut self.reference_id)
    }

    /// Checks if the cursor is on a link inside the doc comments of the given ReferenceId.
    fn find_in_reference_doc_comments(&mut self, id: ReferenceId) {
        let Some(doc_comments) = self.args.interner.doc_comments(id) else {
            return;
        };

        if !doc_comments.iter().any(|doc_comment| self.intersects_span(doc_comment.span())) {
            return;
        }

        let Some((current_module_id, current_type)) = current_module_and_type(id, self.args) else {
            return;
        };

        let Some(byte_lsp_location) = to_lsp_location(
            self.args.files,
            self.args.location.file,
            Span::single_char(self.byte_index as u32),
        ) else {
            return;
        };

        self.link_finder.reset();
        for located_comment in doc_comments {
            let location = located_comment.location();
            let Some(lsp_location) = to_lsp_location(self.args.files, location.file, location.span)
            else {
                continue;
            };
            let start_line = lsp_location.range.start.line;
            let start_char = lsp_location.range.start.character;

            // Read comments from source based on location: the comments in `located_comment` might
            // have been slightly adjusted.
            let comments =
                &self.source[location.span.start() as usize..location.span.end() as usize];

            let links = self.link_finder.find_links(
                comments,
                current_module_id,
                current_type,
                self.args.interner,
                self.args.def_maps,
                self.args.crate_graph,
            );
            for link in links {
                let line = start_line + link.line as u32;
                let start =
                    if link.line == 0 { start_char + link.start as u32 } else { link.start as u32 };
                let length = (link.end - link.start) as u32;
                let end = start + length;
                if byte_lsp_location.range.start.line == line
                    && start <= byte_lsp_location.range.start.character
                    && byte_lsp_location.range.start.character <= end
                {
                    let reference = match link.target {
                        LinkTarget::TopLevelItem(module_def_id) => {
                            module_def_id_to_reference_id(module_def_id)
                        }
                        LinkTarget::Method(_, func_id)
                        | LinkTarget::PrimitiveTypeFunction(_, func_id) => {
                            ReferenceId::Function(func_id)
                        }
                        LinkTarget::StructMember(type_id, index) => {
                            ReferenceId::StructMember(type_id, index)
                        }
                        LinkTarget::PrimitiveType(_) => {
                            continue;
                        }
                    };
                    let location = lsp_types::Location {
                        uri: byte_lsp_location.uri.clone(),
                        range: lsp_types::Range {
                            start: lsp_types::Position { line, character: start },
                            end: lsp_types::Position { line, character: end },
                        },
                    };

                    self.reference_id = Some((reference, Some(location)));
                    return;
                }
            }
        }
    }

    fn intersects_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl Visitor for VisitorReferenceFinder<'_> {
    fn visit_parsed_submodule(&mut self, parsed_sub_module: &ParsedSubModule, _span: Span) -> bool {
        let name_location = parsed_sub_module.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };

        // Switch `self.module_id` to the submodule
        let previous_module_id = self.module_id;

        let def_map = &self.args.def_maps[&self.module_id.krate];
        if let Some(module_data) = def_map.get(self.module_id.local_id) {
            if let Some(child_module) = module_data.children.get(&parsed_sub_module.name) {
                self.module_id = ModuleId { krate: self.module_id.krate, local_id: *child_module };
            }
        }

        parsed_sub_module.accept_children(self);

        // Restore the old module before continuing
        self.module_id = previous_module_id;

        false
    }

    fn visit_noir_function(&mut self, function: &NoirFunction, span: Span) -> bool {
        let name_location = function.name_ident().location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };

        self.intersects_span(span)
    }

    fn visit_noir_struct(&mut self, noir_struct: &NoirStruct, _: Span) -> bool {
        let name_location = noir_struct.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };

        for field in noir_struct.fields.iter() {
            let field_name_location = field.item.name.location();
            if let Some(reference) = self.args.interner.reference_at_location(field_name_location) {
                self.find_in_reference_doc_comments(reference);
            };
        }

        false
    }

    fn visit_noir_enum(&mut self, noir_enum: &NoirEnumeration, _: Span) -> bool {
        let name_location = noir_enum.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };

        for variant in noir_enum.variants.iter() {
            let variant_name_location = variant.item.name.location();
            if let Some(reference) = self.args.interner.reference_at_location(variant_name_location)
            {
                self.find_in_reference_doc_comments(reference);
            };
        }

        false
    }

    fn visit_noir_trait(&mut self, noir_trait: &NoirTrait, _: Span) -> bool {
        let name_location = noir_trait.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };
        true
    }

    fn visit_global(&mut self, let_statement: &LetStatement, _: Span) -> bool {
        let name_location = let_statement.pattern.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };
        false
    }

    fn visit_noir_type_alias(&mut self, type_alias: &TypeAlias, _: Span) -> bool {
        let name_location = type_alias.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };
        false
    }

    /// If the cursor is on an custom attribute, try to resolve its
    /// underlying function and return a ReferenceId to it.
    /// This is needed in hover and go-to-definition because when an annotation generates
    /// code, that code ends up residing in the attribute definition (it ends up having the
    /// attribute's span) so using the usual graph to locate what points to that location
    /// will give not only the attribute function but also any type generated by it.use fm::FileId;
    fn visit_meta_attribute(
        &mut self,
        attribute: &MetaAttribute,
        _target: AttributeTarget,
        span: Span,
    ) -> bool {
        if !self.intersects_span(span) {
            return false;
        }

        let MetaAttributeName::Path(path) = attribute.name.clone() else {
            return false;
        };

        // The path here must resolve to a function and it's a simple path (can't have turbofish)
        // so it can (and must) be solved as an import.
        let Ok(Some((module_def_id, _, _))) = resolve_import(
            path,
            self.module_id,
            self.args.def_maps,
            &mut UsageTracker::default(),
            None, // references tracker
        )
        .map(|result| result.namespace.values) else {
            return true;
        };

        self.reference_id = Some((module_def_id_to_reference_id(module_def_id), None));

        true
    }
}
