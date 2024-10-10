use lsp_types::TextEdit;
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{ConstructorExpression, UnresolvedTypeData},
    node_interner::ReferenceId,
};

use crate::byte_span_to_range;

use super::CodeActionFinder;

impl<'a> CodeActionFinder<'a> {
    pub(super) fn fill_struct_fields(&mut self, constructor: &ConstructorExpression, span: Span) {
        if !self.includes_span(span) {
            return;
        }

        let UnresolvedTypeData::Named(path, _, _) = &constructor.typ.typ else {
            return;
        };

        let location = Location::new(path.span, self.file);
        let Some(ReferenceId::Struct(struct_id)) = self.interner.find_referenced(location) else {
            return;
        };

        let struct_type = self.interner.get_struct(struct_id);
        let struct_type = struct_type.borrow();

        // First get all of the struct's fields
        let mut fields = struct_type.get_fields_as_written();

        // Remove the ones that already exists in the constructor
        for (constructor_field, _) in &constructor.fields {
            fields.retain(|field| field.name.0.contents != constructor_field.0.contents);
        }

        if fields.is_empty() {
            return;
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
            return;
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

        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                new_text.push(',');
                if let Some(line_indent) = &line_indent {
                    new_text.push('\n');
                    new_text.push_str(line_indent);
                } else {
                    new_text.push(' ');
                }
            }
            new_text.push_str(&field.name.0.contents);
            new_text.push_str(": ()");
        }

        if !bytes[right_brace_index - 1].is_ascii_whitespace() {
            new_text.push(' ');
        }

        let title = "Fill struct fields".to_string();
        let text_edit = TextEdit { range, new_text };
        let code_action = self.new_quick_fix(title, text_edit);
        self.code_actions.push(code_action);
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use crate::requests::code_action::tests::assert_code_action;

    #[test]
    async fn test_fill_struct_fields_code_action_no_space() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo {>|<}
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo { one: (), two: () }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_fill_struct_fields_code_action_space() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo { >|<}
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo { one: (), two: () }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_fill_struct_fields_code_action_some_fields() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
            three: Field,
        }

        fn main() {
            Foo { two: 1>|<}
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
            three: Field,
        }

        fn main() {
            Foo { two: 1, one: (), three: () }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_fill_struct_fields_code_action_some_fields_trailing_comma() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
            three: Field,
        }

        fn main() {
            Foo { two: 1,>|<}
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
            three: Field,
        }

        fn main() {
            Foo { two: 1, one: (), three: () }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_fill_struct_fields_code_action_multiline_empty() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo {>|<
            }
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo {
                one: (),
                two: ()
            }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }

    #[test]
    async fn test_fill_struct_fields_code_action_multiline_some_fields() {
        let title = "Fill struct fields";

        let src = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo {>|<
                one: 1,
            }
        }
        "#;

        let expected = r#"
        struct Foo {
            one: Field,
            two: Field,
        }

        fn main() {
            Foo {
                one: 1,
                two: ()
            }
        }
        "#;

        assert_code_action(title, src, expected).await;
    }
}
