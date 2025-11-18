use std::str::FromStr;

use async_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use fm::FileId;
use nargo_doc::links::{LinkFinder, LinkTarget};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    Type,
    ast::{
        GenericTypeArgs, LetStatement, NoirEnumeration, NoirFunction, NoirStruct, NoirTrait, Path,
        TypeAlias, Visitor,
    },
    elaborator::PrimitiveType,
    modules::module_def_id_to_reference_id,
    node_interner::{NodeInterner, ReferenceId},
    parse_program,
    parser::ParsedSubModule,
    signed_field::SignedField,
    token::IntegerTypeSuffix,
};
use num_bigint::BigInt;

use crate::{
    doc_comments::current_module_and_type,
    requests::{
        ProcessRequestCallbackArgs, hover::from_reference::format_reference, to_lsp_location,
    },
    utils,
};

pub(super) fn hover_from_visitor(
    file_id: FileId,
    position: Position,
    args: &ProcessRequestCallbackArgs,
) -> Option<Hover> {
    let file = args.files.get_file(file_id)?;
    let source = file.source();
    let (parsed_module, _errors) = parse_program(source, file_id);
    let byte_index = utils::position_to_byte_index(args.files, file_id, &position)?;

    let mut finder = HoverFinder::new(args, source, file_id, args.interner, byte_index);
    parsed_module.accept(&mut finder);
    finder.hover
}

struct HoverFinder<'a> {
    args: &'a ProcessRequestCallbackArgs<'a>,
    source: &'a str,
    file: FileId,
    interner: &'a NodeInterner,
    byte_index: usize,
    link_finder: LinkFinder,
    hover: Option<Hover>,
}
impl<'a> HoverFinder<'a> {
    fn new(
        args: &'a ProcessRequestCallbackArgs<'a>,
        source: &'a str,
        file: FileId,
        interner: &'a NodeInterner,
        byte_index: usize,
    ) -> Self {
        let link_finder = LinkFinder::default();
        Self { args, source, file, interner, byte_index, link_finder, hover: None }
    }

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

        let Some(byte_lsp_location) =
            to_lsp_location(self.args.files, self.file, Span::single_char(self.byte_index as u32))
        else {
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
                        LinkTarget::PrimitiveType(_) => {
                            continue;
                        }
                    };
                    if let Some(contents) = format_reference(reference, self.args) {
                        let contents = HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: contents,
                        });
                        self.hover = Some(Hover { contents, range: Some(lsp_location.range) });
                    }
                    return;
                }
            }
        }
    }

    fn intersects_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl Visitor for HoverFinder<'_> {
    fn visit_parsed_submodule(&mut self, module: &ParsedSubModule, _: Span) -> bool {
        let name_location = module.name.location();
        if let Some(reference) = self.args.interner.reference_at_location(name_location) {
            self.find_in_reference_doc_comments(reference);
        };

        true
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

    fn visit_literal_integer(
        &mut self,
        value: SignedField,
        _suffix: Option<IntegerTypeSuffix>,
        span: Span,
    ) {
        if !self.intersects_span(span) {
            return;
        }

        let location = Location::new(span, self.file);
        let lsp_location = to_lsp_location(self.args.files, location.file, location.span);
        let range = lsp_location.map(|location| location.range);
        let Some(typ) = self.interner.type_at_location(location) else {
            return;
        };

        // Ignore the suffix when formatting the integer, we already show its type
        let value = format_integer(typ, value);
        let contents = HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value });
        self.hover = Some(Hover { contents, range });
    }

    fn visit_named_type(&mut self, path: &Path, _args: &GenericTypeArgs, span: Span) -> bool {
        // Here we'll try to show docs for primitive types. Non-primitive types are tracked
        // as references, which is handled in `from_reference.rs`.

        if !self.intersects_span(span) {
            return true;
        }

        let Some(ident) = path.as_ident() else {
            return true;
        };

        if !self.intersects_span(ident.span()) {
            return true;
        }

        let location = ident.location();
        if let Some(typ) = self.interner.type_at_location(location) {
            if !typ.is_primitive() {
                return true;
            }
        }

        let name = ident.as_str();
        let Some(markup) = primitive_type_markup_content(name, self.interner) else {
            return true;
        };

        let lsp_location = to_lsp_location(self.args.files, location.file, location.span);
        let range = lsp_location.map(|location| location.range);

        let contents = HoverContents::Markup(markup);
        self.hover = Some(Hover { contents, range });

        false
    }
}

fn format_integer(typ: &Type, value: SignedField) -> String {
    let value_base_10 = value.absolute_value().to_string();

    // For simplicity we parse the value as a BigInt to convert it to hex
    // because `FieldElement::to_hex` will include many leading zeros.
    let value_big_int = BigInt::from_str(&value_base_10).unwrap();
    let negative = if value.is_negative() { "-" } else { "" };

    format!(
        "    {typ}\n---\nvalue of literal: `{negative}{value_base_10} ({negative}0x{value_big_int:02x})`"
    )
}

/// Returns the MarkupContent for the given primitive type name, if the name denotes
/// a primitive type.
fn primitive_type_markup_content(name: &str, interner: &NodeInterner) -> Option<MarkupContent> {
    PrimitiveType::lookup_by_name(name)?;

    let mut value = String::new();
    value.push_str("    ");
    value.push_str(name);

    if let Some(comments) = interner.primitive_docs.get(name) {
        value.push_str("\n---\n");
        for comment in comments {
            value.push_str(&comment.contents);
            value.push('\n');
        }
    }

    Some(MarkupContent { kind: MarkupKind::Markdown, value })
}

#[cfg(test)]
mod tests {
    use noirc_frontend::{
        Type, ast::IntegerBitSize, shared::Signedness, signed_field::SignedField,
    };

    use super::format_integer;

    #[test]
    fn format_integer_zero() {
        let typ = Type::FieldElement;
        let value = SignedField::positive(0_u128);
        let expected = "    Field\n---\nvalue of literal: `0 (0x00)`";
        assert_eq!(format_integer(&typ, value), expected);
    }

    #[test]
    fn format_positive_integer() {
        let typ = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
        let value = SignedField::positive(123456_u128);
        let expected = "    u32\n---\nvalue of literal: `123456 (0x1e240)`";
        assert_eq!(format_integer(&typ, value), expected);
    }

    #[test]
    fn format_negative_integer() {
        let typ = Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour);
        let value = SignedField::new(987654_u128.into(), true);
        let expected = "    i64\n---\nvalue of literal: `-987654 (-0xf1206)`";
        assert_eq!(format_integer(&typ, value), expected);
    }
}
