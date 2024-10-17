use noirc_frontend::{
    ast::{
        ArrayLiteral, BinaryOpKind, BlockExpression, CallExpression, CastExpression,
        ConstructorExpression, Expression, ExpressionKind, IfExpression, IndexExpression,
        InfixExpression, Lambda, Literal, MemberAccessExpression, MethodCallExpression,
        PrefixExpression, TypePath, UnaryOp, UnresolvedTypeData,
    },
    token::{Keyword, Token},
};

use super::{
    chunks::{Chunk, ChunkTag, Chunks, TextChunk},
    Formatter,
};

impl<'a> Formatter<'a> {
    pub(super) fn format_expression(&mut self, expression: Expression, chunks: &mut Chunks) {
        chunks.leading_comment(self.skip_comments_and_whitespace_chunk());

        match expression.kind {
            ExpressionKind::Literal(literal) => self.format_literal(literal, chunks),
            ExpressionKind::Block(block) => {
                chunks.group(self.format_block_expression(
                    block, false, // force multiple lines
                ));
            }
            ExpressionKind::Prefix(prefix_expression) => {
                chunks.group(self.format_prefix(*prefix_expression));
            }
            ExpressionKind::Index(index_expression) => {
                chunks.group(self.format_index_expression(*index_expression))
            }
            ExpressionKind::Call(call) => chunks.group(self.format_call(*call)),
            ExpressionKind::MethodCall(method_call) => {
                chunks.group(self.format_method_call(*method_call))
            }
            ExpressionKind::Constructor(constructor) => {
                chunks.group(self.format_constructor(*constructor));
            }
            ExpressionKind::MemberAccess(member_access) => {
                chunks.group(self.format_member_access(*member_access));
            }
            ExpressionKind::Cast(cast_expression) => {
                chunks.group(self.format_cast(*cast_expression));
            }
            ExpressionKind::Infix(infix_expression) => {
                chunks.group(self.format_infix_expression(*infix_expression))
            }
            ExpressionKind::If(if_expression) => {
                chunks.group(self.format_if_expression(
                    *if_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::Variable(path) => {
                chunks.text(self.chunk(|formatter| {
                    formatter.format_path(path);
                }));
            }
            ExpressionKind::Tuple(exprs) => chunks.group(self.format_tuple(exprs)),
            ExpressionKind::Lambda(lambda) => chunks.group(self.format_lambda(*lambda)),
            ExpressionKind::Parenthesized(expression) => {
                chunks.group(self.format_parenthesized_expression(*expression));
            }
            ExpressionKind::Quote(..) => {
                chunks.group(self.format_quote());
            }
            ExpressionKind::Unquote(..) => {
                unreachable!("Should not be present in the AST")
            }
            ExpressionKind::Comptime(block_expression, _span) => {
                chunks.group(self.format_comptime_expression(
                    block_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::Unsafe(block_expression, _span) => {
                chunks.group(self.format_unsafe_expression(
                    block_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::AsTraitPath(as_trait_path) => {
                chunks.text(self.chunk(|formatter| formatter.format_as_trait_path(as_trait_path)))
            }
            ExpressionKind::TypePath(type_path) => {
                chunks.group(self.format_type_path(type_path));
            }
            ExpressionKind::Resolved(..)
            | ExpressionKind::Interned(..)
            | ExpressionKind::InternedStatement(..)
            | ExpressionKind::Error => unreachable!("Should not be present in the AST"),
        }
    }

    fn format_literal(&mut self, literal: Literal, chunks: &mut Chunks) {
        match literal {
            Literal::Unit => chunks.text(self.chunk(|formatter| {
                formatter.write_left_paren();
                formatter.write_right_paren();
            })),
            Literal::Bool(_) | Literal::Str(_) | Literal::FmtStr(_) | Literal::RawStr(..) => chunks
                .text(self.chunk(|formatter| {
                    formatter.write_current_token_as_in_source();
                    formatter.bump();
                })),
            Literal::Integer(..) => chunks.text(self.chunk(|formatter| {
                if formatter.token == Token::Minus {
                    formatter.write_token(Token::Minus);
                    formatter.skip_comments_and_whitespace();
                }
                formatter.write_current_token_as_in_source();
                formatter.bump();
            })),
            Literal::Array(array_literal) => chunks.group(self.format_array_literal(
                array_literal,
                false, // is slice
            )),
            Literal::Slice(array_literal) => {
                chunks.group(self.format_array_literal(
                    array_literal,
                    true, // is slice
                ))
            }
        }
    }

    fn format_array_literal(&mut self, literal: ArrayLiteral, is_slice: bool) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.one_chunk_per_line = false;

        chunks.text(self.chunk(|formatter| {
            if is_slice {
                formatter.write_token(Token::Ampersand);
            }
            formatter.write_left_bracket();
        }));

        match literal {
            ArrayLiteral::Standard(exprs) => {
                self.format_expressions_separated_by_comma(
                    exprs,
                    false, // force trailing comma
                    &mut chunks,
                );
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                chunks.increase_indentation();
                chunks.line();

                self.format_expression(*repeated_element, &mut chunks);
                chunks.text(self.chunk(|formatter| {
                    formatter.write_semicolon();
                    formatter.write_space();
                }));
                self.format_expression(*length, &mut chunks);

                chunks.decrease_indentation();
                chunks.line();
            }
        }

        chunks.text(self.chunk(|formatter| formatter.write_right_bracket()));

        chunks
    }

    fn format_tuple(&mut self, exprs: Vec<Expression>) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.one_chunk_per_line = false;
        let force_trailing_comma = exprs.len() == 1;

        chunks.text(self.chunk(|formatter| {
            formatter.write_left_paren();
        }));

        self.format_expressions_separated_by_comma(exprs, force_trailing_comma, &mut chunks);

        chunks.text(self.chunk(|formatter| formatter.write_right_paren()));

        chunks
    }

    fn format_lambda(&mut self, lambda: Lambda) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Pipe);
        }));
        self.format_items_separated_by_comma(
            lambda.parameters,
            false, // force trailing comma
            false, // surround with spaces
            &mut chunks,
            |formatter, (pattern, typ), chunks| {
                chunks.text(formatter.chunk(|formatter| {
                    formatter.format_pattern(pattern);
                    if typ.typ != UnresolvedTypeData::Unspecified {
                        formatter.write_token(Token::Colon);
                        formatter.write_space();
                        formatter.format_type(typ);
                    }
                }));
            },
        );
        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Pipe);
            formatter.write_space();
            if lambda.return_type.typ != UnresolvedTypeData::Unspecified {
                formatter.write_token(Token::Arrow);
                formatter.write_space();
                formatter.format_type(lambda.return_type);
                formatter.write_space();
            }
        }));
        self.format_expression(lambda.body, &mut chunks);
        chunks
    }

    fn format_parenthesized_expression(&mut self, expr: Expression) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_left_paren();
        }));
        self.format_expression(expr, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));
        chunks
    }

    pub(super) fn format_quote(&mut self) -> Chunks {
        // We use the current token rather than the Tokens we got from `Token::Quote` because
        // the current token has whitespace and comments in it, while the one we got from
        // the parser doesn't.
        let Token::Quote(tokens) = self.bump() else {
            panic!("Expected current token to be Quote");
        };

        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write("quote {");
            for token in tokens.0 {
                formatter.write_source_span(token.to_span());
            }
            formatter.write("}");
        }));
        chunks
    }

    pub(super) fn format_comptime_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Comptime);
            formatter.write_space();
        }));
        chunks.group(self.format_block_expression(block, force_multiple_lines));
        chunks
    }

    pub(super) fn format_unsafe_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Unsafe);
            formatter.write_space();
        }));
        chunks.group(self.format_block_expression(block, force_multiple_lines));
        chunks
    }

    pub(super) fn format_type_path(&mut self, type_path: TypePath) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.format_type(type_path.typ);
            formatter.write_token(Token::DoubleColon);
            formatter.write_identifier(type_path.item);
            if !type_path.turbofish.is_empty() {
                formatter.write_token(Token::DoubleColon);
                formatter.format_generic_type_args(type_path.turbofish);
            }
        }));
        chunks
    }

    pub(super) fn format_expressions_separated_by_comma(
        &mut self,
        exprs: Vec<Expression>,
        force_trailing_comma: bool,
        chunks: &mut Chunks,
    ) {
        self.format_items_separated_by_comma(
            exprs,
            force_trailing_comma,
            false, // surround with spaces
            chunks,
            |formatter, expr, chunks| {
                formatter.format_expression(expr, chunks);
            },
        );
    }

    pub(super) fn format_items_separated_by_comma<Item, F>(
        &mut self,
        items: Vec<Item>,
        force_trailing_comma: bool,
        surround_with_spaces: bool,
        mut chunks: &mut Chunks,
        mut format_item: F,
    ) where
        F: FnMut(&mut Self, Item, &mut Chunks),
    {
        chunks.increase_indentation();
        if surround_with_spaces {
            chunks.space_or_line();
        } else {
            chunks.line();
        }

        for (index, expr) in items.into_iter().enumerate() {
            if index > 0 {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_comma();
                }));
                chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());
                chunks.space_or_line();
            }
            format_item(self, expr, &mut chunks);
        }

        let chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();

            // Trailing comma
            if formatter.token == Token::Comma {
                formatter.bump();
                formatter.skip_comments_and_whitespace();
            }
        });

        // Make sure to put a trailing comma before the last parameter comments, if there were any
        if force_trailing_comma {
            chunks.text(TextChunk::new(",".to_string()));
        } else {
            chunks.text_if_multiline(TextChunk::new(",".to_string()));
        }
        chunks.text(chunk);

        chunks.decrease_indentation();
        if surround_with_spaces {
            chunks.space_or_line();
        } else {
            chunks.line();
        }
    }

    fn format_constructor(&mut self, constructor: ConstructorExpression) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.format_type(constructor.typ);
            formatter.write_space();
            formatter.write_left_brace();
        }));

        if constructor.fields.is_empty() {
            self.format_empty_block_contents();
        } else {
            self.format_items_separated_by_comma(
                constructor.fields,
                false, // force trailing comma
                true,  // surround with spaces
                &mut chunks,
                |formatter, (name, value), chunks| {
                    chunks.text(formatter.chunk(|formatter| {
                        formatter.write_identifier(name);
                        formatter.skip_comments_and_whitespace();
                    }));

                    if formatter.token == Token::Colon {
                        chunks.text(formatter.chunk(|formatter| {
                            formatter.write_token(Token::Colon);
                            formatter.write_space();
                        }));
                        formatter.format_expression(value, chunks);
                    }
                },
            );
        }
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_brace();
        }));

        chunks
    }

    fn format_member_access(&mut self, member_access: MemberAccessExpression) -> Chunks {
        let mut chunks = Chunks::new();

        self.format_expression(member_access.lhs, &mut chunks);

        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Dot);
            formatter.write_identifier(member_access.rhs);
        }));

        chunks
    }

    fn format_cast(&mut self, cast_expression: CastExpression) -> Chunks {
        let mut chunks = Chunks::new();
        self.format_expression(cast_expression.lhs, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_space();
            formatter.write_keyword(Keyword::As);
            formatter.write_space();
            formatter.format_type(cast_expression.r#type);
        }));
        chunks
    }

    fn format_prefix(&mut self, prefix: PrefixExpression) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            if let UnaryOp::MutableReference = prefix.operator {
                formatter.write_current_token();
                formatter.bump();
                formatter.skip_comments_and_whitespace();
                formatter.write_current_token();
                formatter.bump();
                formatter.write_space();
            } else {
                formatter.write_current_token();
                formatter.bump();
            }
        }));
        self.format_expression(prefix.rhs, &mut chunks);
        chunks
    }

    fn format_infix_expression(&mut self, infix: InfixExpression) -> Chunks {
        let mut chunks = Chunks::new();

        self.format_expression(infix.lhs, &mut chunks);
        chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());

        chunks.increase_indentation();
        chunks.space_or_line();
        chunks.text(self.chunk(|formatter| {
            let tokens_count =
                if infix.operator.contents == BinaryOpKind::ShiftRight { 2 } else { 1 };
            for _ in 0..tokens_count {
                formatter.write_current_token();
                formatter.bump();
            }
            formatter.write_space();
        }));

        self.format_expression(infix.rhs, &mut chunks);

        chunks
    }

    pub(super) fn format_if_expression(
        &mut self,
        if_expression: IfExpression,
        mut force_multiple_lines: bool,
    ) -> Chunks {
        let mut chunks = Chunks::new();

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::If);
            formatter.write_space();
        }));

        self.format_expression(if_expression.condition, &mut chunks);

        let comment_chunk_after_condition = self.skip_comments_and_whitespace_chunk();
        if comment_chunk_after_condition.has_newlines {
            force_multiple_lines = true;
            chunks.trailing_comment(comment_chunk_after_condition);
        } else {
            chunks.text(self.chunk(|formatter| {
                formatter.write_space();
            }));
        }

        let ExpressionKind::Block(consequence_block) = if_expression.consequence.kind else {
            panic!("Expected if expression consequence to be a block");
        };

        if let Some(alternative) = &if_expression.alternative {
            match &alternative.kind {
                ExpressionKind::Block(block) => {
                    if block.statements.len() > 1 {
                        force_multiple_lines = true;
                    }
                }
                ExpressionKind::If(..) => {
                    force_multiple_lines = true;
                }
                _ => panic!("Unexpected if alternative expression kind"),
            }
        }

        let mut consequence_group =
            self.format_block_expression(consequence_block, force_multiple_lines);
        consequence_group.tag = Some(ChunkTag::IfConsequenceOrAlternative);
        chunks.group(consequence_group);

        if let Some(alternative) = if_expression.alternative {
            chunks.text(self.chunk(|formatter| {
                formatter.write_space();
                formatter.write_keyword(Keyword::Else);
            }));

            let comment_chunk_after_else = self.skip_comments_and_whitespace_chunk();
            if comment_chunk_after_else.has_newlines {
                force_multiple_lines = true;
                chunks.trailing_comment(comment_chunk_after_else);
            } else {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_space();
                }));
            }

            let mut alternative_group = match alternative.kind {
                ExpressionKind::Block(block) => {
                    self.format_block_expression(block, force_multiple_lines)
                }
                ExpressionKind::If(if_expression) => {
                    self.format_if_expression(*if_expression, force_multiple_lines)
                }
                _ => panic!("Unexpected if alternative expression kind"),
            };

            alternative_group.tag = Some(ChunkTag::IfConsequenceOrAlternative);
            chunks.group(alternative_group);
        }

        if force_multiple_lines || chunks.width() > self.config.single_line_if_else_max_width {
            force_if_chunks_to_multiple_lines(&mut chunks);
        }

        chunks
    }

    fn format_index_expression(&mut self, index: IndexExpression) -> Chunks {
        let mut chunks = Chunks::new();
        self.format_expression(index.collection, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_left_bracket();
        }));
        self.format_expression(index.index, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_bracket();
        }));
        chunks
    }

    fn format_call(&mut self, call: CallExpression) -> Chunks {
        let mut chunks = Chunks::new();

        self.format_expression(*call.func, &mut chunks);

        chunks.text(self.chunk(|formatter| {
            if call.is_macro_call {
                formatter.write_token(Token::Bang);
            }
            formatter.write_left_paren();
        }));
        self.format_expressions_separated_by_comma(
            call.arguments,
            false, // force trailing comma
            &mut chunks,
        );
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));

        chunks
    }

    fn format_method_call(&mut self, method_call: MethodCallExpression) -> Chunks {
        let mut chunks = Chunks::new();

        self.format_expression(method_call.object, &mut chunks);

        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Dot);
            formatter.write_identifier(method_call.method_name);
            if method_call.is_macro_call {
                formatter.write_token(Token::Bang);
            }
            if let Some(generics) = method_call.generics {
                formatter.format_turbofish(generics);
            }
            formatter.write_left_paren();
        }));
        self.format_expressions_separated_by_comma(
            method_call.arguments,
            false, // force trailing comma
            &mut chunks,
        );
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));

        chunks
    }

    pub(super) fn format_block_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.text(self.chunk(|formatter| {
            formatter.write_left_brace();
        }));
        self.format_block_expression_contents(block, force_multiple_lines, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_right_brace();
        }));
        chunks
    }

    pub(super) fn format_block_expression_contents(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
        mut chunks: &mut Chunks,
    ) {
        if block.is_empty() {
            chunks.increase_indentation();
            chunks.leading_comment(self.skip_comments_and_whitespace_chunk());
            chunks.decrease_indentation();
        } else {
            self.format_non_empty_block_expression_contents(
                block,
                force_multiple_lines,
                &mut chunks,
            );
        }
    }

    pub(super) fn format_non_empty_block_expression_contents(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
        mut chunks: &mut Chunks,
    ) {
        chunks.force_multiple_lines = force_multiple_lines || block.statements.len() > 1;
        let surround_with_spaces = !chunks.force_multiple_lines && block.statements.len() == 1;

        chunks.increase_indentation();
        if surround_with_spaces {
            chunks.space_or_line();
        } else {
            chunks.line();
        }

        for (index, statement) in block.statements.into_iter().enumerate() {
            if index > 0 {
                let count = self.following_newlines_count();
                if count > 0 {
                    // If newlines follow, we first add a line, then add the comment chunk
                    chunks.lines(count > 1);
                    chunks.leading_comment(self.skip_comments_and_whitespace_chunk());
                } else {
                    // Otherwise, add the comment first as it's a trailing comment
                    chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());
                    chunks.line();
                }
            }

            self.format_statement(statement, &mut chunks);
        }

        chunks.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();
        }));

        chunks.decrease_indentation();

        if surround_with_spaces {
            chunks.space_or_line();
        } else {
            chunks.line();
        }
    }

    pub(super) fn format_empty_block_contents(&mut self) {
        let mut chunks = Chunks::new();
        chunks.increase_indentation();
        let chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace_writing_lines_if_found();
        });

        if chunk.string.trim().is_empty() {
            // We only found whitespace until the next non-whitespace-non-comment token,
            // so there's nothing to write.
            return;
        } else {
            // There were comments, so we have to write those indented, then write
            // a final newline in case we found a `//` comment.
            chunks.text(chunk);
            chunks.decrease_indentation();
            chunks.line();
            self.format_chunks(chunks);
        }
    }
}

fn force_if_chunks_to_multiple_lines(chunks: &mut Chunks) {
    // Note: what if we have something like this?
    //
    // ```
    // if foo { if bar { } }
    // ```
    //
    // and we determine the outer if needs to be formatted in multiple lines?
    // Because we apply this to all "if" chunks, we'll also mark the inner
    // if to be formatted in multiples lines... but this is fine. At least
    // rustfmt will not format the above if in a single line (if there's
    // an if inside an if it forces the outer if to be formatted in multiple lines).
    if let Some(ChunkTag::IfConsequenceOrAlternative) = chunks.tag {
        chunks.force_multiple_lines = true;
    }

    for chunk in chunks.chunks.iter_mut() {
        if let Chunk::Group(group) = chunk {
            force_if_chunks_to_multiple_lines(group);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_format, assert_format_with_config, assert_format_with_max_width, Config};

    #[test]
    fn format_unit() {
        let src = "global x =  ( ) ;";
        let expected = "global x = ();\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_false() {
        let src = "global x =  false ;";
        let expected = "global x = false;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_true() {
        let src = "global x =  true ;";
        let expected = "global x = true;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_integer() {
        let src = "global x =  42 ;";
        let expected = "global x = 42;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_negative_integer() {
        let src = "global x =  - 42 ;";
        let expected = "global x = -42;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_ref_mut_integer() {
        let src = "global x = & mut 42 ;";
        let expected = "global x = &mut 42;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_hex_integer() {
        let src = "global x =  0xff ;";
        let expected = "global x = 0xff;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_string() {
        let src = "global x =  \"hello\" ;";
        let expected = "global x = \"hello\";\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_fmtstr() {
        let src = "global x =  f\"hello\" ;";
        let expected = "global x = f\"hello\";\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_standard_array() {
        let src = "global x = [ 1 , 2 , 3 , ] ;";
        let expected = "global x = [1, 2, 3];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_standard_slice() {
        let src = "global x = & [ 1 , 2 , 3 , ] ;";
        let expected = "global x = &[1, 2, 3];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_repeated_array() {
        let src = "global x = [ 1 ; 3 ] ;";
        let expected = "global x = [1; 3];\n";
        assert_format(src, expected);
    }

    // TODO: this is maybe not ideal
    #[test]
    fn format_long_array_in_global() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, ] ;";
        let expected = "global x = [
    1, 2, 3, 4, 5,
];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_long_array_in_global_in_mod() {
        let src = "mod moo { mod bar { global x = [ 1 , 2 , 3 , 4, 5, ] ; } }";
        let expected = "mod moo {
    mod bar {
        global x = [
            1, 2, 3, 4,
            5,
        ];
    }
}
";
        assert_format_with_max_width(src, expected, 25);
    }

    // TODO: this is missing a newline between the globals
    #[test]
    fn format_long_array_in_global_2() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, ] ;

global y = 1;
        ";
        let expected = "global x = [
    1, 2, 3, 4, 5,
];
global y = 1;
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_very_long_array_in_global() {
        let src = "global x = [ 1 , 2 , 3 , 4, 5, 6, 789, 123, 234, 345] ;";
        let expected = "global x = [
    1, 2, 3, 4, 5, 6,
    789, 123, 234, 345,
];
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_array_in_global_with_line_comments() {
        let src = "global x = [ // hello
        1 , 2 ] ;";
        let expected = "global x = [
    // hello
    1, 2,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_2() {
        let src = "global x = [ // hello
         [ 1 , 2 ]  ] ;";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_3() {
        let src = "global x =
    [ 
        // hello
        [1, 2],  
    ];
";
        let expected = "global x = [
    // hello
    [1, 2],
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_line_comments_4() {
        let src = "global x =
    [
        1, // world 
        2, 3,
    ];
";
        let expected = "global x = [
    1, // world
    2, 3,
];
";
        assert_format(src, expected);
    }

    #[test]
    fn format_array_in_global_with_block_comments() {
        let src = "global x = [ /* hello */
        1 , 2 ] ;";
        let expected = "global x = [
    /* hello */
    1, 2,
];
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_cast() {
        let src = "global x =  1  as  u8 ;";
        let expected = "global x = 1 as u8;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_variable() {
        let src = "global x =  y ;";
        let expected = "global x = y;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple() {
        let src = "global x = ( 1 , 2 , 3 , ) ;";
        let expected = "global x = (1, 2, 3);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple_length_one() {
        let src = "global x = ( 1 , ) ;";
        let expected = "global x = (1,);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_as_trait_path() {
        let src = "global x = < i32 as foo > :: bar ;";
        let expected = "global x = <i32 as foo>::bar;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_index() {
        let src = "global x = foo [ bar ] ;";
        let expected = "global x = foo[bar];\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_long_index() {
        let src = "global x = foo [ bar [ baz [ qux [ one [ two ]]]] ] ; global y = 1;";
        let expected = "global x = foo[bar[baz[
    qux[one[two]]]]];
global y = 1;
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_long_index_2() {
        let src = "global x = foo [ bar ] [ baz ] [ qux ] [ one ] [ two ] ; global y = 1;";
        let expected = "global x = foo[bar][baz]
    [qux][one][two];
global y = 1;
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_prefix() {
        let src = "global x = - a ;";
        let expected = "global x = -a;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_infix() {
        let src = "global x =  a  +  b  ;";
        let expected = "global x = a + b;\n";
        assert_format(src, expected);
    }

    // TODO: this is not ideal
    #[test]
    fn format_long_infix_same_operator() {
        let src = "global x =  one + two + three + four + five ;";
        let expected = "global x = one + two
    + three
        + four
            + five;
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_empty_block() {
        let src = "global x =  {  }  ;";
        let expected = "global x = {};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_with_one_statement() {
        let src = "global x =  {  1  }  ;";
        let expected = "global x = { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_block_with_two_statements() {
        let src = "global x =  {  1; 2  }  ;";
        let expected = "global x = {
    1;
    2
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_call() {
        let src = "global x =  foo :: bar ( 1, 2 )  ;";
        let expected = "global x = foo::bar(1, 2);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_call_with_turbofish() {
        let src = "global x =  foo :: bar :: < Field, i32 > ( 1, 2 )  ;";
        let expected = "global x = foo::bar::<Field, i32>(1, 2);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call() {
        let src = "global x =  bar . baz ( 1, 2 )  ;";
        let expected = "global x = bar.baz(1, 2);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call_with_generics() {
        let src = "global x =  bar . baz :: < T >  ( 1, 2 )  ;";
        let expected = "global x = bar.baz::<T>(1, 2);\n";
        assert_format(src, expected);
    }

    // TODO: this is not ideal
    #[test]
    fn format_method_call_chain() {
        let src = "global x =  bar . baz ( 1, 2 ) . qux ( 1 , 2, 3) . one ( 5, 6)  ;";
        let expected = "global x = bar.baz(
    1,
    2,
).qux(
    1,
    2,
    3,
).one(
    5,
    6,
);
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_member_access() {
        let src = "global x =  bar . baz   ;";
        let expected = "global x = bar.baz;\n";
        assert_format(src, expected);
    }

    // TODO: this is not ideal
    #[test]
    fn format_long_member_access() {
        let src = "global x =  foo . bar . baz . qux . this_is_a_long_name   ;";
        let expected = "global x = foo.bar
    .baz.qux
        .this_is_a_long_name;
";
        assert_format_with_max_width(src, expected, 20);
    }

    #[test]
    fn format_parenthesized() {
        let src = "global x =  ( 1 )   ;";
        let expected = "global x = (1);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_unsafe_one_expression() {
        let src = "global x = unsafe { 1  } ;";
        let expected = "global x = unsafe { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_unsafe_two_expressions() {
        let src = "global x = unsafe { 1; 2  } ;";
        let expected = "global x = unsafe {
    1;
    2
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_comptime_one_expression() {
        let src = "global x = comptime { 1  } ;";
        let expected = "global x = comptime { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_comptime_two_expressions() {
        let src = "global x = comptime { 1; 2  } ;";
        let expected = "global x = comptime {
    1;
    2
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_empty_constructor() {
        let src = "global x = Foo { } ;";
        let expected = "global x = Foo {};\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_constructor() {
        let src = "global x = Foo { one: 1 , two : 2 , three } ;";
        let expected = "global x = Foo { one: 1, two: 2, three };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_type_path() {
        let src = "global x = Field :: max  ;";
        let expected = "global x = Field::max;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_type_path_with_turbofish() {
        let src = "global x = Field :: max :: < i32 > ;";
        let expected = "global x = Field::max::<i32>;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_expression_without_else_one_expression() {
        let src = "global x = if  1   {   2   } ;";
        let expected = "global x = if 1 { 2 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_expression_without_else_two_expressions() {
        let src = "global x = if  1   {   2; 3   } ;";
        let expected = "global x = if 1 {
    2;
    3
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_expression_with_else() {
        let src = "global x = if  1   {   2   }  else  {  3  };";
        let expected = "global x = if 1 { 2 } else { 3 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_expression_with_else_multiple_exprs() {
        let src = "global x = if  1   {   2   }  else  {  3; 4  };";
        let expected = "global x = if 1 {
    2
} else {
    3;
    4
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_expression_else_if() {
        let src = "global x = if  1   {   2   }  else if 3 {  4  };";
        let expected = "global x = if 1 {
    2
} else if 3 {
    4
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_with_configurable_maximum_if_width() {
        let src = "global x = if  123   {   456   }  else  {  789  };";
        let expected = "global x = if 123 {
    456
} else {
    789
};\n";

        let config = Config {
            single_line_if_else_max_width: "if 123 { 456 } else { 789 }".len() - 1,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_if_with_configurable_maximum_if_width_2() {
        let src = "global x = if  foo(123)   {   456   }  else  {  789  };";
        let expected = "global x = if foo(123) {
    456
} else {
    789
};\n";

        let config = Config {
            single_line_if_else_max_width: "if foo(123) { 456 } else { 789 }".len() - 1,
            ..Config::default()
        };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_if_with_comment_after_condition() {
        let src = "global x = if  123  // some comment  
        {   456   }  ;";
        let expected = "global x = if 123 // some comment
{
    456
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_if_with_comment_after_else() {
        let src = "global x = if  123  {   456   } else  // some comment 
        { 789 };";
        let expected = "global x = if 123 {
    456
} else // some comment
{
    789
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_quote() {
        let src = "global x = quote { 1  2  3 $four $(five) };";
        let expected = "global x = quote { 1  2  3 $four $(five) };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_no_parameters() {
        let src = "global x = | |  1 ;";
        let expected = "global x = || 1;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_parameters() {
        let src = "global x = | x , y : Field , z |  1 ;";
        let expected = "global x = |x, y: Field, z| 1;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_block() {
        let src = "global x = | |  {  1  } ;";
        let expected = "global x = || { 1 };\n";
        assert_format(src, expected);
    }
}
