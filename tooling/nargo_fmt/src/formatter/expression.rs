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
    chunks::{Chunk, ChunkKind, ChunkTag, Chunks, TextChunk},
    Formatter,
};

#[derive(Debug)]
struct FormattedLambda {
    group: Chunks,
    first_line_width: usize,
}

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
            ExpressionKind::Lambda(lambda) => chunks.group(self.format_lambda(*lambda).group),
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
        chunks.kind = ChunkKind::ExpressionList;

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
        chunks.kind = ChunkKind::ExpressionList;

        let force_trailing_comma = exprs.len() == 1;

        chunks.text(self.chunk(|formatter| {
            formatter.write_left_paren();
        }));

        self.format_expressions_separated_by_comma(exprs, force_trailing_comma, &mut chunks);

        chunks.text(self.chunk(|formatter| formatter.write_right_paren()));

        chunks
    }

    fn format_lambda(&mut self, lambda: Lambda) -> FormattedLambda {
        let mut chunks = Chunks::new();

        let params_and_return_type_chunk = self.chunk(|formatter| {
            formatter.write_token(Token::Pipe);
            for (index, (pattern, typ)) in lambda.parameters.into_iter().enumerate() {
                if index > 0 {
                    formatter.write_comma();
                    formatter.write_space();
                }
                formatter.format_pattern(pattern);
                if typ.typ != UnresolvedTypeData::Unspecified {
                    formatter.write_token(Token::Colon);
                    formatter.write_space();
                    formatter.format_type(typ);
                }
            }
            formatter.skip_comments_and_whitespace();
            if formatter.token == Token::Comma {
                formatter.bump();
            }
            formatter.write_token(Token::Pipe);
            formatter.write_space();
            if lambda.return_type.typ != UnresolvedTypeData::Unspecified {
                formatter.write_token(Token::Arrow);
                formatter.write_space();
                formatter.format_type(lambda.return_type);
                formatter.write_space();
            }
        });

        let params_and_return_type_chunk_width = params_and_return_type_chunk.width;

        chunks.text(params_and_return_type_chunk);

        let body_is_block = matches!(lambda.body.kind, ExpressionKind::Block(..));

        let width_before_body = chunks.width();

        self.format_expression(lambda.body, &mut chunks);

        let width_after_body = chunks.width();

        let first_line_width = params_and_return_type_chunk_width
            + (if body_is_block {
                // 1 because we already have `|param1, param2, ..., paramN| ` (including the space)
                // so all that's left is a `{`.
                1
            } else {
                // The body is not a block so we can't assume it'll go into multiple lines
                width_after_body - width_before_body
            });

        FormattedLambda { group: chunks, first_line_width }
    }

    fn format_parenthesized_expression(&mut self, expr: Expression) -> Chunks {
        let is_nested_parenthesized = matches!(expr.kind, ExpressionKind::Parenthesized(..));

        let mut chunks = Chunks::new();
        let left_paren_chunk = self.chunk(|formatter| {
            formatter.write_left_paren();
        });

        let mut group = Chunks::new();
        let mut has_comments = false;

        let comment_after_left_paren_chunk = self.skip_comments_and_whitespace_chunk();
        if !comment_after_left_paren_chunk.string.trim().is_empty() {
            has_comments = true;
        }

        group.leading_comment(comment_after_left_paren_chunk);

        self.format_expression(expr, &mut group);

        let comment_before_right_parent_chunk = self.skip_comments_and_whitespace_chunk();
        if !comment_before_right_parent_chunk.string.trim().is_empty() {
            has_comments = true;
        }

        let right_paren_chunk = self.chunk(|formatter| {
            formatter.write_right_paren();
        });

        if is_nested_parenthesized && !has_comments && self.config.remove_nested_parens {
            chunks.chunks.extend(group.chunks);
        } else {
            chunks.text(left_paren_chunk);
            chunks.increase_indentation();
            chunks.line();
            chunks.chunks.extend(group.chunks);
            chunks.text(comment_before_right_parent_chunk);
            chunks.decrease_indentation();
            chunks.line();
            chunks.text(right_paren_chunk);
        }

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
        if exprs.is_empty() {
            if let Some(group) = self.empty_block_contents_chunk() {
                chunks.group(group);
            }
        } else {
            let exprs_len = exprs.len();
            let mut expr_index = 0;

            self.format_items_separated_by_comma(
                exprs,
                force_trailing_comma,
                false, // surround with spaces
                chunks,
                |formatter, expr, chunks| {
                    // If the last expression in the list is a lambda, we format it but we mark
                    // the chunk in a special way: it likely has newlines, but we don't want
                    // those newlines to affect the parent group. For example:
                    //
                    //     foo(1, 2, |x| {
                    //       let y = x + 1;
                    //       y * 2
                    //     })
                    if expr_index == exprs_len - 1 {
                        if let ExpressionKind::Lambda(lambda) = expr.kind {
                            let mut lambda_group = formatter.format_lambda(*lambda);
                            lambda_group.group.kind = ChunkKind::LambdaAsLastExpressionInList {
                                first_line_width: lambda_group.first_line_width,
                            };
                            chunks.group(lambda_group.group);
                            return;
                        }
                    }
                    expr_index += 1;

                    formatter.format_expression(expr, chunks);
                },
            );
        }
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
        let mut comments_chunk = self.skip_comments_and_whitespace_chunk();

        // If the comment is not empty but doesn't have newlines, it's surely `/* comment */`.
        // We format that with spaces surrounding it so it looks, for example, like `Foo { /* comment */ field ..`.
        if !comments_chunk.string.trim().is_empty() && !comments_chunk.has_newlines {
            // Note: there's no space after `{}` because space will be produced by format_items_separated_by_comma
            comments_chunk.string = if surround_with_spaces {
                format!(" {}", comments_chunk.string.trim())
            } else {
                format!(" {} ", comments_chunk.string.trim())
            };
            chunks.text(comments_chunk);

            chunks.increase_indentation();
            if surround_with_spaces {
                chunks.space_or_line();
            } else {
                chunks.line();
            }
        } else {
            chunks.increase_indentation();
            if surround_with_spaces {
                chunks.space_or_line();
            } else {
                chunks.line();
            }

            chunks.trailing_comment(comments_chunk);
        }

        for (index, expr) in items.into_iter().enumerate() {
            if index > 0 {
                chunks.text_attached_to_last_group(self.chunk(|formatter| {
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
        if !force_trailing_comma {
            chunks.trailing_comma();
        }

        chunks.text(chunk);

        if force_trailing_comma {
            chunks.text(TextChunk::new(",".to_string()));
        }

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
            if let Some(group) = self.empty_block_contents_chunk() {
                chunks.group(group);
            }
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
        // Keep track of how much indentation increased by formatting the member access.
        // At the end we'll decrease the indentation by that amount.
        let mut increased_indentation = 0;

        let mut chunks = self.format_member_access_impl(member_access, &mut increased_indentation);

        // Decrease the indentation if it increased.
        for _ in 0..increased_indentation {
            chunks.decrease_indentation();
        }

        chunks
    }

    fn format_member_access_impl(
        &mut self,
        member_access: MemberAccessExpression,
        increased_indentation: &mut usize,
    ) -> Chunks {
        let mut chunks = Chunks::new();

        // If we have code like `foo.bar.baz.qux`, where `member_access.lhs` is also a MemberAccessExpression,
        // we'll format it with the same tag. Once the lhs is not a MemberAccessExpression, we'll format it
        // and add an increase in indentation, but just once so that it ends up being formatted like this
        // in case it needs to be formatted in multiple lines:
        //
        //    foo.bar
        //        .baz
        //        .qux
        //
        // Note that we do the same if the lhs is a MethodCallExpression.
        //
        // Also note that we don't format it like this:
        //
        //    foo
        //        .bar
        //        .baz
        //        .qux
        //
        // For that, we check if the lhs'lhs is also a MemberAccess/MethodCall to determine where we need
        // to put a line and an indentation.
        let mut increase_indentation = false;

        // Write a `line()` before the dot?
        let mut line_before_dot = false;

        match member_access.lhs.kind {
            ExpressionKind::MemberAccess(lhs_member_access) => {
                let lhs_lhs_is_member_access_or_call = matches!(
                    lhs_member_access.lhs.kind,
                    ExpressionKind::MemberAccess(..) | ExpressionKind::MethodCall(..)
                );

                // If we have `foo.bar.baz.qux`
                //             ^~~~~~~~~~~      --> lhs
                //             ^~~~~~~          --> lhs.lhs
                // and lhs.lhs is a member access or call, we don't want to add an extra indent.
                //
                // Otherwise, it's something like this `foo.bar.baz` so we increase the
                // indentation after `foo.bar`.
                if !lhs_lhs_is_member_access_or_call {
                    increase_indentation = true;
                }

                // We always put a line before the dot if lhs is a member access or call
                line_before_dot = true;

                chunks.group(
                    self.format_member_access_impl(*lhs_member_access, increased_indentation),
                );
            }
            ExpressionKind::MethodCall(lhs_method_call) => {
                let lhs_lhs_is_member_access_or_call = matches!(
                    lhs_method_call.object.kind,
                    ExpressionKind::MemberAccess(..) | ExpressionKind::MethodCall(..)
                );

                // If we have `foo.bar.baz.qux`
                //             ^~~~~~~~~~~      --> lhs
                //             ^~~~~~~          --> lhs.lhs
                // and lhs.lhs is a member access or call, we don't want to add an extra indent.
                //
                // Otherwise, it's something like this `foo.bar.baz` so we increase the
                // indentation after `foo.bar`.
                if !lhs_lhs_is_member_access_or_call {
                    increase_indentation = true;
                }

                // We always put a line before the dot if lhs is a member access or call
                line_before_dot = true;

                chunks.group(self.format_method_call_impl(
                    *lhs_method_call,
                    increased_indentation,
                    true, // nested
                ));
            }
            _ => {
                self.format_expression(member_access.lhs, &mut chunks);
            }
        };

        chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());

        if increase_indentation {
            chunks.increase_indentation();
            *increased_indentation += 1;
        }

        if line_before_dot {
            chunks.line();
        }

        chunks.text(self.chunk(|formatter| {
            formatter.write_token(Token::Dot);
            formatter.write_identifier_or_integer(member_access.rhs);
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
        let chunk_tag = self.next_chunk_tag();

        // Keep track of how much indentation increased by formatting the infix expression.
        // At the end we'll decrease the indentation by that amount.
        let mut increased_indentation = 0;
        let mut chunks = self.format_infix_expression_with_chunk_tag(
            infix,
            chunk_tag,
            &mut increased_indentation,
        );
        chunks.force_multiline_on_children_with_same_tag_if_multiline = true;

        // Decrease the indentation if it increased.
        for _ in 0..increased_indentation {
            chunks.decrease_indentation();
        }

        chunks
    }

    fn format_infix_expression_with_chunk_tag(
        &mut self,
        infix: InfixExpression,
        chunk_tag: ChunkTag,
        increased_indentation: &mut usize,
    ) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.tag = Some(chunk_tag);

        // If we have code like `a + b + c + d`, that's always parsed as `((a + b) + c) + d` where each
        // parentheses denotes an InfixExpression. So, if the lhs of the current infix expression is also
        // an infix expression with the same operator, we format it with the same tag.
        // If the lhs is not an infix expression or has a different operator, we format it normally,
        // and afterwards signal an increase in indentation. That way if this infix expression has
        // to be formatted in multiple lines, we'll only indent after the first operand
        // (we still produce "space or line" after each operator).
        let increase_indentation = match infix.lhs.kind {
            ExpressionKind::Infix(lhs_infix) if lhs_infix.operator == infix.operator => {
                chunks.group(self.format_infix_expression_with_chunk_tag(
                    *lhs_infix,
                    chunk_tag,
                    increased_indentation,
                ));
                false
            }
            _ => {
                self.format_expression(infix.lhs, &mut chunks);
                true
            }
        };

        let mut comment_chunk_after_lhs = self.skip_comments_and_whitespace_chunk();

        // If the comment is not empty but doesn't have newlines, it's surely `/* comment */`.
        // We format that with spaces surrounding it so it looks like `a /* comment */ + b`.
        if !comment_chunk_after_lhs.string.trim().is_empty()
            && !comment_chunk_after_lhs.has_newlines
        {
            // Note: there's no space after `{}` because a bit below comes "space_or_line".
            comment_chunk_after_lhs.string = format!(" {}", comment_chunk_after_lhs.string.trim());
            chunks.text(comment_chunk_after_lhs);
        } else {
            chunks.trailing_comment(comment_chunk_after_lhs);
        }

        if increase_indentation {
            chunks.increase_indentation();
            *increased_indentation += 1;
        }

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
        let chunk_tag = self.next_chunk_tag();
        let mut chunks = self.format_if_expression_with_chunk_tag(
            if_expression,
            &mut force_multiple_lines,
            chunk_tag,
        );

        if force_multiple_lines || chunks.width() > self.config.single_line_if_else_max_width {
            force_if_chunks_to_multiple_lines(&mut chunks, chunk_tag);
        }

        chunks
    }

    pub(super) fn format_if_expression_with_chunk_tag(
        &mut self,
        if_expression: IfExpression,
        force_multiple_lines: &mut bool,
        chunk_tag: ChunkTag,
    ) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.tag = Some(chunk_tag);

        chunks.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::If);
            formatter.write_space();
        }));

        self.format_expression(if_expression.condition, &mut chunks);

        let comment_chunk_after_condition = self.skip_comments_and_whitespace_chunk();
        if comment_chunk_after_condition.has_newlines {
            *force_multiple_lines = true;
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
                        *force_multiple_lines = true;
                    }
                }
                ExpressionKind::If(..) => {
                    *force_multiple_lines = true;
                }
                _ => panic!("Unexpected if alternative expression kind"),
            }
        }

        let mut consequence_group =
            self.format_block_expression(consequence_block, *force_multiple_lines);
        consequence_group.tag = Some(chunk_tag);
        chunks.group(consequence_group);

        if let Some(alternative) = if_expression.alternative {
            chunks.text(self.chunk(|formatter| {
                formatter.write_space();
                formatter.write_keyword(Keyword::Else);
            }));

            let comment_chunk_after_else = self.skip_comments_and_whitespace_chunk();
            if comment_chunk_after_else.has_newlines {
                *force_multiple_lines = true;
                chunks.trailing_comment(comment_chunk_after_else);
            } else {
                chunks.text(self.chunk(|formatter| {
                    formatter.write_space();
                }));
            }

            let mut alternative_group = match alternative.kind {
                ExpressionKind::Block(block) => {
                    self.format_block_expression(block, *force_multiple_lines)
                }
                ExpressionKind::If(if_expression) => self.format_if_expression_with_chunk_tag(
                    *if_expression,
                    force_multiple_lines,
                    chunk_tag,
                ),
                _ => panic!("Unexpected if alternative expression kind"),
            };

            alternative_group.tag = Some(chunk_tag);
            chunks.group(alternative_group);
        }

        chunks
    }

    fn format_index_expression(&mut self, index: IndexExpression) -> Chunks {
        let mut chunks = Chunks::new();
        self.format_expression(index.collection, &mut chunks);
        chunks.text(self.chunk(|formatter| {
            formatter.write_left_bracket();
        }));

        // If we have:
        //
        //     foo[ // bar
        //       1]
        //
        // and there were newlines in the comment section, we format it like this:
        //
        //     foo[
        //       // bar
        //       1
        //     ]
        //
        // That is, we first put a newline before the comment so it looks a bit better.
        // This is a rare scenario, but we had a test for this before the formatter was
        // rewritten, so...
        let comments_chunk = self.skip_comments_and_whitespace_chunk();
        let comments_chunk_has_newlines = comments_chunk.has_newlines;

        if comments_chunk_has_newlines {
            chunks.increase_indentation();
            chunks.line();
        }

        chunks.leading_comment(comments_chunk);

        self.format_expression(index.index, &mut chunks);

        if comments_chunk_has_newlines {
            chunks.decrease_indentation();
            chunks.line();
        }

        chunks.text(self.chunk(|formatter| {
            formatter.write_right_bracket();
        }));
        chunks
    }

    fn format_call(&mut self, call: CallExpression) -> Chunks {
        let mut chunks = Chunks::new();
        chunks.kind = ChunkKind::ExpressionList;

        self.format_expression(*call.func, &mut chunks);

        chunks.text(self.chunk(|formatter| {
            if call.is_macro_call {
                formatter.write_token(Token::Bang);
            }
            formatter.write_left_paren();
        }));

        // Format arguments in a separate group so we can calculate the arguments
        // width and determine if we need to format this call in multiple lines.
        let mut group = Chunks::new();
        self.format_expressions_separated_by_comma(
            call.arguments,
            false, // force trailing comma
            &mut group,
        );

        if group.width() > self.config.fn_call_width {
            chunks.force_multiple_lines = true;
        }

        // We no longer need this subgroup, so put all its chunks into the main chunks
        chunks.chunks.extend(group.chunks);

        chunks.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));

        chunks
    }

    fn format_method_call(&mut self, method_call: MethodCallExpression) -> Chunks {
        // Keep track of how much indentation increased by formatting the method call.
        // At the end we'll decrease the indentation by that amount.
        let mut increased_indentation = 0;
        let mut chunks = self.format_method_call_impl(
            method_call,
            &mut increased_indentation,
            false, // nested
        );

        // Decrease the indentation if it increased.
        for _ in 0..increased_indentation {
            chunks.decrease_indentation();
        }

        chunks
    }

    fn format_method_call_impl(
        &mut self,
        method_call: MethodCallExpression,
        increased_indentation: &mut usize,
        nested: bool,
    ) -> Chunks {
        let mut chunks = Chunks::new();

        // The logic here is similar to that of `format_member_access_with_chunk_tag`, so
        // please that function inner comments for details.
        let mut increase_indentation_before_dot = false;
        let mut increase_arguments_indentation = false;

        // Write a `line()` before the dot?
        let mut line_before_dot = false;

        match method_call.object.kind {
            ExpressionKind::MethodCall(lhs_method_call) => {
                let lhs_lhs_is_member_access_or_call = matches!(
                    lhs_method_call.object.kind,
                    ExpressionKind::MemberAccess(..) | ExpressionKind::MethodCall(..)
                );

                if !lhs_lhs_is_member_access_or_call {
                    increase_indentation_before_dot = true;
                }

                line_before_dot = true;

                chunks.group(self.format_method_call_impl(
                    *lhs_method_call,
                    increased_indentation,
                    true, // nested
                ));
            }
            ExpressionKind::MemberAccess(lhs_member_access) => {
                let lhs_lhs_is_member_access_or_call = matches!(
                    lhs_member_access.lhs.kind,
                    ExpressionKind::MemberAccess(..) | ExpressionKind::MethodCall(..)
                );

                if !lhs_lhs_is_member_access_or_call {
                    increase_indentation_before_dot = true;
                }

                line_before_dot = true;

                chunks.group(
                    self.format_member_access_impl(*lhs_member_access, increased_indentation),
                );
            }
            _ => {
                self.format_expression(method_call.object, &mut chunks);

                // If we have `foo.bar(..)` where `lhs` is neither a member access nor a call,
                // but this occurs inside another member access or method call we are formatting, like
                // `foo.bar(..).baz` , then if we end up formatting all of this in multiple lines
                // we want to have an extra level of indentation in the arguments, so it formats like this:
                //
                //     foo.bar(
                //         1,   // Note how we indented twice
                //         2,
                //         3,
                //     )
                //     .baz
                if nested {
                    increase_arguments_indentation = true;
                }
            }
        }

        chunks.trailing_comment(self.skip_comments_and_whitespace_chunk());

        if increase_indentation_before_dot {
            chunks.increase_indentation();
            *increased_indentation += 1;
        }

        if line_before_dot {
            chunks.line();
        }

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

        if increase_arguments_indentation {
            chunks.increase_indentation();
        }

        let mut group = Chunks::new();
        group.kind = ChunkKind::ExpressionList;
        self.format_expressions_separated_by_comma(
            method_call.arguments,
            false, // force trailing comma
            &mut group,
        );
        chunks.group(group);

        if increase_arguments_indentation {
            chunks.decrease_indentation();
        }

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
            if let Some(block_chunks) = self.empty_block_contents_chunk() {
                chunks.chunks.extend(block_chunks.chunks);
            }
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
        if let Some(chunks) = self.empty_block_contents_chunk() {
            self.format_chunks(chunks);
        }
    }

    pub(super) fn empty_block_contents_chunk(&mut self) -> Option<Chunks> {
        let mut chunks = Chunks::new();
        chunks.increase_indentation();
        let mut chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace_writing_lines_if_found();
        });

        if chunk.string.trim().is_empty() {
            // We only found whitespace until the next non-whitespace-non-comment token,
            // so there's nothing to write.
            None
        } else {
            if chunk.string.trim_start().starts_with("//") {
                chunks.text(chunk);
                chunks.decrease_indentation();
                chunks.line();
            } else {
                chunk.string = format!(" {} ", chunk.string.trim());
                chunks.text(chunk);
                chunks.decrease_indentation();
            }
            Some(chunks)
        }
    }
}

fn force_if_chunks_to_multiple_lines(chunks: &mut Chunks, chunk_tag: ChunkTag) {
    if chunks.tag == Some(chunk_tag) {
        chunks.force_multiple_lines = true;
    }

    for chunk in chunks.chunks.iter_mut() {
        if let Chunk::Group(group) = chunk {
            force_if_chunks_to_multiple_lines(group, chunk_tag);
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

    #[test]
    fn format_long_infix_same_operator_1() {
        let src = "global x =  one + two + three;";
        let expected = "global x = one
    + two
    + three;
";
        assert_format_with_max_width(src, expected, "one + two + three".len() - 1);
    }

    #[test]
    fn format_long_infix_same_operator_2() {
        let src = "global x =  one + two + three + four;";
        let expected = "global x = one
    + two
    + three
    + four;
";
        assert_format_with_max_width(src, expected, "one + two + three + four".len() - 1);
    }

    #[test]
    fn format_long_infix_same_operator_3() {
        let src = "fn foo() { one + two + three + four }";
        let expected = "fn foo() {
    one
        + two
        + three
        + four
}
";
        assert_format_with_max_width(src, expected, "one + two + three + four".len() - 1);
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
    fn format_call_with_maximum_width() {
        let src = "global x =  foo :: bar ( 1, 2, 3 )  ;";
        let expected = "global x = foo::bar(
    1,
    2,
    3,
);\n";

        let config = Config { fn_call_width: "1, 2, 3".len() - 1, ..Default::default() };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_call_with_maximum_width_2() {
        let src = "global x =  foo::bar::baz( );";
        let expected = "global x = foo::bar::baz();\n";
        assert_format_with_max_width(src, expected, "foo::bar::baz".len() - 1);
    }

    #[test]
    fn format_call_with_maximum_width_comma_exceeds() {
        let src = "global x = foo::bar(
    baz::qux(1, 2, 3),
);";
        let expected = "global x = foo::bar(
    baz::qux(
        1,
        2,
        3,
    ),
);
";
        assert_format_with_max_width(src, expected, "    baz::qux(1, 2, 3),".len() - 1);
    }

    #[test]
    fn format_call_with_maximum_width_comma_exceeds_2() {
        let src = "global x = foo::bar(
    |x, y| { some_chunk_of_code },
);";
        let expected = "global x = foo::bar(
    |x, y| {
        some_chunk_of_code
    },
);
";
        assert_format_with_max_width(src, expected, "    |x, y| { some_chunk_of_code },".len() - 1);
    }

    #[test]
    fn format_nested_call_max_width() {
        let src = "global _callStackItem1 = context.call_public_function(
            context.this_address(),
            comptime {
            FunctionSelector::from_signature(\"broadcast(Field)\")
        },
            [owner]
        );";
        let expected = "global _callStackItem1 = context.call_public_function(
    context.this_address(),
    comptime { FunctionSelector::from_signature(\"broadcast(Field)\") },
    [owner],
);
";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call() {
        let src = "global x =  bar . baz ( 1, 2 )  ;";
        let expected = "global x = bar.baz(1, 2);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call_with_long_arguments() {
        let src = "global x =  bar . baz ( 123456789, 123456789, 123456789, 123456789, 123456789, 123456789, 123456789 )  ;";
        let expected = "global x = bar.baz(
    123456789,
    123456789,
    123456789,
    123456789,
    123456789,
    123456789,
    123456789,
);
";
        assert_format_with_max_width(src, expected, 40);
    }

    #[test]
    fn format_method_call_with_generics() {
        let src = "global x =  bar . baz :: < T >  ( 1, 2 )  ;";
        let expected = "global x = bar.baz::<T>(1, 2);\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call_chain() {
        let src = "global x =  bar . baz ( 1, 2 ) . qux ( 1 , 2, 3) . one ( 5, 6)  ;";
        let expected = "global x = bar.baz(1, 2)
    .qux(1, 2, 3)
    .one(5, 6);
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_method_call_with_maximum_width() {
        let src = "global x =  foo::bar.baz( );";
        let expected = "global x = foo::bar.baz();\n";
        assert_format_with_max_width(src, expected, "foo::bar.baz".len() - 1);
    }

    #[test]
    fn format_member_access() {
        let src = "global x =  bar . baz   ;";
        let expected = "global x = bar.baz;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_long_member_access_alone() {
        let src = "global x =  foo . bar . baz . qux . final   ;";
        let expected = "global x = foo.bar
    .baz
    .qux
    .final;
";
        assert_format_with_max_width(src, expected, "foo.bar.baz.qux.final".len() - 1);
    }

    #[test]
    fn format_long_member_access_and_method_call_chain() {
        let src = "global x =  foo . bar(1, 2) . baz . qux(2, 3) . this_is_a_long_name   ;";
        let expected = "global x = foo.bar(1, 2)
    .baz
    .qux(2, 3)
    .this_is_a_long_name;
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_long_member_access_and_method_call_chain_2() {
        let src = "fn burn() {
    storage.at(from).sub(from_keys.npk_m, U128::from_integer(amount))
    .emit(encode_and_encrypt_note());  
}
";
        let expected = "fn burn() {
    storage.at(from).sub(from_keys.npk_m, U128::from_integer(amount))
        .emit(encode_and_encrypt_note());
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_tuple_member_access() {
        let src = "global x =  bar . 0   ;";
        let expected = "global x = bar.0;\n";
        assert_format(src, expected);
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
    fn format_constructor_with_turbofish() {
        let src = "global x = Foo :: < Bar > { one } ;";
        let expected = "global x = Foo::<Bar> { one };\n";
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

    #[test]
    fn format_lambda_with_block_multiple_statements() {
        let src = "global x = | a, b |  {  1; 2  } ;";
        let expected = "global x = |a, b| {
    1;
    2
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_call_argument() {
        let src = "global x = foo(1, |x| { 1; 2 });";
        let expected = "global x = foo(1, |x| {
    1;
    2
});
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_method_call_argument() {
        let src = "global x = foo.bar(1, |x| { 1; 2 });";
        let expected = "global x = foo.bar(1, |x| {
    1;
    2
});
";
        assert_format(src, expected);
    }

    #[test]
    fn removes_nested_parens() {
        let src = "global x = ( ( ( ( ) ) ) ) ;";
        let expected = "global x = (());\n";
        assert_format(src, expected);
    }

    #[test]
    fn does_not_remove_nested_parens_if_not_told_so() {
        let src = "global x = ( ( ( ( ) ) ) ) ;";
        let expected = "global x = (((())));\n";

        let config = Config { remove_nested_parens: false, ..Config::default() };
        assert_format_with_config(src, expected, config);
    }
}
