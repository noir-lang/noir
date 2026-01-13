use std::str::FromStr;

use async_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use fm::FileId;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    Type,
    ast::{GenericTypeArgs, Path, Visitor},
    elaborator::PrimitiveType,
    node_interner::NodeInterner,
    parse_program,
    signed_field::SignedField,
    token::IntegerTypeSuffix,
};
use num_bigint::BigInt;

use crate::{
    requests::{ProcessRequestCallbackArgs, to_lsp_location},
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

    let mut finder = HoverFinder::new(args, file_id, args.interner, byte_index);
    parsed_module.accept(&mut finder);
    finder.hover
}

struct HoverFinder<'a> {
    args: &'a ProcessRequestCallbackArgs<'a>,
    file: FileId,
    interner: &'a NodeInterner,
    byte_index: usize,
    hover: Option<Hover>,
}
impl<'a> HoverFinder<'a> {
    fn new(
        args: &'a ProcessRequestCallbackArgs<'a>,
        file: FileId,
        interner: &'a NodeInterner,
        byte_index: usize,
    ) -> Self {
        Self { args, file, interner, byte_index, hover: None }
    }

    fn intersects_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl Visitor for HoverFinder<'_> {
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
