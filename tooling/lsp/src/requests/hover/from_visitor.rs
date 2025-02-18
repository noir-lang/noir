use std::str::FromStr;

use acvm::FieldElement;
use fm::{FileId, FileMap};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use noirc_errors::{Location, Span};
use noirc_frontend::{ast::Visitor, node_interner::NodeInterner, parse_program, Type};
use num_bigint::BigInt;

use crate::{
    requests::{to_lsp_location, ProcessRequestCallbackArgs},
    utils,
};

pub(super) fn hover_from_visitor(
    file_id: Option<FileId>,
    position: Position,
    args: &ProcessRequestCallbackArgs,
) -> Option<Hover> {
    let file_id = file_id?;
    let file = args.files.get_file(file_id)?;
    let source = file.source();
    let (parsed_module, _errors) = parse_program(source);
    let byte_index = utils::position_to_byte_index(args.files, file_id, &position)?;

    let mut finder = HoverFinder::new(args.files, file_id, args.interner, byte_index);
    parsed_module.accept(&mut finder);
    finder.hover
}

struct HoverFinder<'a> {
    files: &'a FileMap,
    file: FileId,
    interner: &'a NodeInterner,
    byte_index: usize,
    hover: Option<Hover>,
}
impl<'a> HoverFinder<'a> {
    fn new(
        files: &'a FileMap,
        file: FileId,
        interner: &'a NodeInterner,
        byte_index: usize,
    ) -> Self {
        Self { files, file, interner, byte_index, hover: None }
    }

    fn intersects_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}

impl<'a> Visitor for HoverFinder<'a> {
    fn visit_literal_integer(&mut self, value: FieldElement, negative: bool, span: Span) {
        if !self.intersects_span(span) {
            return;
        }

        let location = Location::new(span, self.file);
        let lsp_location = to_lsp_location(self.files, location.file, location.span);
        let range = lsp_location.map(|location| location.range);
        let Some(typ) = self.interner.type_at_location(location) else {
            return;
        };

        let value = format_integer(typ, value, negative);
        let contents = HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value });
        self.hover = Some(Hover { contents, range });
    }
}

fn format_integer(typ: Type, value: FieldElement, negative: bool) -> String {
    let value_base_10 = value.to_string();

    // For simplicity we parse the value as a BigInt to convert it to hex
    // because `FieldElement::to_hex` will include many leading zeros.
    let value_big_int = BigInt::from_str(&value_base_10).unwrap();
    let negative = if negative { "-" } else { "" };

    format!("    {typ}\n---\nvalue of literal: `{negative}{value_base_10} ({negative}0x{value_big_int:02x})`")
}

#[cfg(test)]
mod tests {
    use noirc_frontend::{
        ast::{IntegerBitSize, Signedness},
        Type,
    };

    use super::format_integer;

    #[test]
    fn format_integer_zero() {
        let typ = Type::FieldElement;
        let value = 0_u128.into();
        let negative = false;
        let expected = "    Field\n---\nvalue of literal: `0 (0x00)`";
        assert_eq!(format_integer(typ, value, negative), expected);
    }

    #[test]
    fn format_positive_integer() {
        let typ = Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo);
        let value = 123456_u128.into();
        let negative = false;
        let expected = "    u32\n---\nvalue of literal: `123456 (0x1e240)`";
        assert_eq!(format_integer(typ, value, negative), expected);
    }

    #[test]
    fn format_negative_integer() {
        let typ = Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour);
        let value = 987654_u128.into();
        let negative = true;
        let expected = "    i64\n---\nvalue of literal: `-987654 (-0xf1206)`";
        assert_eq!(format_integer(typ, value, negative), expected);
    }
}
