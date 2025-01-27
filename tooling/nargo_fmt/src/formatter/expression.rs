use noirc_frontend::{
    ast::{
        ArrayLiteral, BinaryOpKind, BlockExpression, CallExpression, CastExpression,
        ConstructorExpression, Expression, ExpressionKind, IfExpression, IndexExpression,
        InfixExpression, Lambda, Literal, MemberAccessExpression, MethodCallExpression,
        PrefixExpression, TypePath, UnaryOp, UnresolvedTypeData,
    },
    token::{Keyword, Token},
};

use crate::chunks::{Chunk, ChunkFormatter, ChunkGroup, GroupKind, GroupTag, TextChunk};

use super::Formatter;

#[derive(Debug)]
struct FormattedLambda {
    group: ChunkGroup,
    first_line_width: usize,
}

impl<'a, 'b> ChunkFormatter<'a, 'b> {
    pub(super) fn format_expression(&mut self, expression: Expression, group: &mut ChunkGroup) {
        group.leading_comment(self.skip_comments_and_whitespace_chunk());

        match expression.kind {
            ExpressionKind::Literal(literal) => self.format_literal(literal, group),
            ExpressionKind::Block(block) => {
                group.group(self.format_block_expression(
                    block, false, // force multiple lines
                ));
            }
            ExpressionKind::Prefix(prefix_expression) => {
                group.group(self.format_prefix(*prefix_expression));
            }
            ExpressionKind::Index(index_expression) => {
                group.group(self.format_index_expression(*index_expression));
            }
            ExpressionKind::Call(call) => group.group(self.format_call(*call)),
            ExpressionKind::MethodCall(method_call) => {
                group.group(self.format_method_call(*method_call));
            }
            ExpressionKind::Constructor(constructor) => {
                group.group(self.format_constructor(*constructor));
            }
            ExpressionKind::MemberAccess(member_access) => {
                group.group(self.format_member_access(*member_access));
            }
            ExpressionKind::Cast(cast_expression) => {
                group.group(self.format_cast(*cast_expression));
            }
            ExpressionKind::Infix(infix_expression) => {
                group.group(self.format_infix_expression(*infix_expression));
            }
            ExpressionKind::If(if_expression) => {
                group.group(self.format_if_expression(
                    *if_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::Variable(path) => {
                group.text(self.chunk(|formatter| {
                    formatter.format_path(path);
                }));
            }
            ExpressionKind::Tuple(exprs) => group.group(self.format_tuple(exprs)),
            ExpressionKind::Lambda(lambda) => group.group(self.format_lambda(*lambda).group),
            ExpressionKind::Parenthesized(expression) => {
                group.group(self.format_parenthesized_expression(*expression));
            }
            ExpressionKind::Quote(..) => {
                group.group(self.format_quote());
            }
            ExpressionKind::Unquote(..) => {
                unreachable!("Should not be present in the AST")
            }
            ExpressionKind::Comptime(block_expression, _span) => {
                group.group(self.format_comptime_expression(
                    block_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::Unsafe(block_expression, _span) => {
                group.group(self.format_unsafe_expression(
                    block_expression,
                    false, // force multiple lines
                ));
            }
            ExpressionKind::AsTraitPath(as_trait_path) => {
                group.text(self.chunk(|formatter| formatter.format_as_trait_path(as_trait_path)));
            }
            ExpressionKind::TypePath(type_path) => {
                group.group(self.format_type_path(type_path));
            }
            ExpressionKind::Resolved(..)
            | ExpressionKind::Interned(..)
            | ExpressionKind::InternedStatement(..)
            | ExpressionKind::Error => unreachable!("Should not be present in the AST"),
        }
    }

    fn format_literal(&mut self, literal: Literal, group: &mut ChunkGroup) {
        match literal {
            Literal::Unit => group.text(self.chunk(|formatter| {
                formatter.write_left_paren();
                formatter.write_right_paren();
            })),
            Literal::Bool(_) | Literal::Str(_) | Literal::FmtStr(_, _) | Literal::RawStr(..) => {
                group.text(self.chunk(|formatter| {
                    formatter.write_current_token_as_in_source();
                    formatter.bump();
                }));
            }
            Literal::Integer(..) => group.text(self.chunk(|formatter| {
                if formatter.is_at(Token::Minus) {
                    formatter.write_token(Token::Minus);
                    formatter.skip_comments_and_whitespace();
                }
                formatter.write_current_token_as_in_source();
                formatter.bump();
            })),
            Literal::Array(array_literal) => group.group(self.format_array_literal(
                array_literal,
                false, // is slice
            )),
            Literal::Slice(array_literal) => {
                group.group(self.format_array_literal(
                    array_literal,
                    true, // is slice
                ));
            }
        }
    }

    fn format_array_literal(&mut self, literal: ArrayLiteral, is_slice: bool) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        group.text(self.chunk(|formatter| {
            if is_slice {
                formatter.write_token(Token::Ampersand);
            }
            formatter.write_left_bracket();
        }));

        match literal {
            ArrayLiteral::Standard(exprs) => {
                group.kind = GroupKind::ExpressionList {
                    prefix_width: group.width(),
                    expressions_count: exprs.len(),
                };

                let maximum_element_width = self.format_expressions_separated_by_comma(
                    exprs, false, // force trailing comma
                    &mut group,
                );
                group.one_chunk_per_line =
                    maximum_element_width > self.config.short_array_element_width_threshold;
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                group.increase_indentation();
                group.line();

                self.format_expression(*repeated_element, &mut group);
                group.semicolon(self);
                group.space(self);
                self.format_expression(*length, &mut group);

                group.decrease_indentation();
                group.line();
            }
        }

        group.text(self.chunk(|formatter| formatter.write_right_bracket()));

        group
    }

    fn format_tuple(&mut self, exprs: Vec<Expression>) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.one_chunk_per_line = false;

        let force_trailing_comma = exprs.len() == 1;

        group.text(self.chunk(|formatter| {
            formatter.write_left_paren();
        }));

        group.kind = GroupKind::ExpressionList {
            prefix_width: group.width(),
            expressions_count: exprs.len(),
        };

        self.format_expressions_separated_by_comma(exprs, force_trailing_comma, &mut group);

        group.text(self.chunk(|formatter| formatter.write_right_paren()));

        group
    }

    fn format_lambda(&mut self, lambda: Lambda) -> FormattedLambda {
        let mut group = ChunkGroup::new();

        let lambda_has_return_type = lambda.return_type.typ != UnresolvedTypeData::Unspecified;

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
            if formatter.is_at(Token::Comma) {
                formatter.bump();
            }
            formatter.write_token(Token::Pipe);
            formatter.write_space();
            if lambda_has_return_type {
                formatter.write_token(Token::Arrow);
                formatter.write_space();
                formatter.format_type(lambda.return_type);
                formatter.write_space();
            }
        });

        let params_and_return_type_chunk_width = params_and_return_type_chunk.width;

        group.text(params_and_return_type_chunk);

        let block_statement_count = if let ExpressionKind::Block(block) = &lambda.body.kind {
            Some(block.statements.len())
        } else {
            None
        };

        let mut body_group = ChunkGroup::new();

        let comments_count_before_body = self.written_comments_count;
        self.format_expression(lambda.body, &mut body_group);

        body_group.kind = GroupKind::LambdaBody {
            block_statement_count,
            has_comments: self.written_comments_count > comments_count_before_body,
            lambda_has_return_type,
        };

        group.group(body_group);

        let first_line_width = params_and_return_type_chunk_width
            + (if block_statement_count.is_some() {
                // 1 because we already have `|param1, param2, ..., paramN| ` (including the space)
                // so all that's left is a `{`.
                1
            } else {
                // The body is not a block so we can write it right away
                0
            });

        FormattedLambda { group, first_line_width }
    }

    fn format_parenthesized_expression(&mut self, expr: Expression) -> ChunkGroup {
        let is_nested_parenthesized = matches!(expr.kind, ExpressionKind::Parenthesized(..));

        let mut group = ChunkGroup::new();
        let left_paren_chunk = self.chunk(|formatter| {
            formatter.write_left_paren();
        });

        let mut expr_group = ChunkGroup::new();
        let mut has_comments = false;

        let comment_after_left_paren_chunk = self.skip_comments_and_whitespace_chunk();
        if !comment_after_left_paren_chunk.string.trim().is_empty() {
            has_comments = true;
        }

        expr_group.leading_comment(comment_after_left_paren_chunk);

        self.format_expression(expr, &mut expr_group);

        let comment_before_right_parent_chunk = self.skip_comments_and_whitespace_chunk();
        if !comment_before_right_parent_chunk.string.trim().is_empty() {
            has_comments = true;
        }

        let right_paren_chunk = self.chunk(|formatter| {
            formatter.write_right_paren();
        });

        if is_nested_parenthesized && !has_comments && self.config.remove_nested_parens {
            group.chunks.extend(expr_group.chunks);
        } else {
            group.text(left_paren_chunk);
            group.increase_indentation();
            group.line();
            group.chunks.extend(expr_group.chunks);
            group.text(comment_before_right_parent_chunk);
            group.decrease_indentation();
            group.line();
            group.text(right_paren_chunk);
        }

        group
    }

    pub(super) fn format_quote(&mut self) -> ChunkGroup {
        // A quote's prefix isn't captured in the token, so let's figure it out which one
        // is it by looking at the source code.
        let mut quote_source_code =
            &self.source[self.token_span.start() as usize..self.token_span.end() as usize];

        // Remove "quote" and any whitespace following it
        quote_source_code = quote_source_code.strip_prefix("quote").unwrap();
        quote_source_code = quote_source_code.trim_start();

        // The first char is the delimiter
        let delimiter_start = quote_source_code.chars().next().unwrap();
        let delimiter_end = match delimiter_start {
            '(' => ')',
            '{' => '}',
            '[' => ']',
            _ => panic!("Unexpected delimiter: {}", delimiter_start),
        };

        // We use the current token rather than the Tokens we got from `Token::Quote` because
        // the current token has whitespace and comments in it, while the one we got from
        // the parser doesn't.
        let Token::Quote(tokens) = self.bump() else {
            panic!("Expected current token to be Quote");
        };

        let mut group = ChunkGroup::new();
        group.verbatim(self.chunk(|formatter| {
            formatter.write("quote");
            formatter.write_space();
            formatter.write(&delimiter_start.to_string());
            for token in tokens.0 {
                formatter.write_source_span(token.to_span());
            }
            formatter.write(&delimiter_end.to_string());
        }));
        group
    }

    pub(super) fn format_comptime_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::Comptime);
            formatter.write_space();
        }));
        group.group(self.format_block_expression(block, force_multiple_lines));
        group
    }

    pub(super) fn format_unsafe_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
            formatter.format_outer_doc_comments();
            formatter.write_keyword(Keyword::Unsafe);
            formatter.write_space();
        }));
        group.group(self.format_block_expression(block, force_multiple_lines));
        group
    }

    pub(super) fn format_type_path(&mut self, type_path: TypePath) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
            formatter.format_type(type_path.typ);
            formatter.write_token(Token::DoubleColon);
            formatter.write_identifier(type_path.item);
            if let Some(turbofish) = type_path.turbofish {
                formatter.write_token(Token::DoubleColon);
                formatter.format_generic_type_args(turbofish);
            }
        }));
        group
    }

    /// Returns the maximum width of each expression to format. For example,
    /// if the list is [1, 234, 56], the maximum width is 3 (that of `234`).
    pub(super) fn format_expressions_separated_by_comma(
        &mut self,
        exprs: Vec<Expression>,
        force_trailing_comma: bool,
        group: &mut ChunkGroup,
    ) -> usize {
        if exprs.is_empty() {
            if let Some(inner_group) = self.empty_block_contents_chunk() {
                group.group(inner_group);
            }
            0
        } else {
            let exprs_len = exprs.len();
            let mut expr_index = 0;
            let mut max_width = 0;

            self.format_items_separated_by_comma(
                exprs,
                force_trailing_comma,
                false, // surround with spaces
                group,
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
                            lambda_group.group.kind = GroupKind::LambdaAsLastExpressionInList {
                                first_line_width: lambda_group.first_line_width,
                                indentation: None,
                            };
                            chunks.group(lambda_group.group);
                            return;
                        }
                    }
                    expr_index += 1;

                    let chunks_len_before_expression = chunks.chunks.len();

                    formatter.format_expression(expr, chunks);

                    let chunks_len_after_expression = chunks.chunks.len();
                    let expression_width: usize = (chunks_len_before_expression
                        ..chunks_len_after_expression)
                        .map(|index| chunks.chunks[index].width())
                        .sum();
                    if expression_width > max_width {
                        max_width = expression_width;
                    }
                },
            );

            max_width
        }
    }

    pub(super) fn format_items_separated_by_comma<Item, F>(
        &mut self,
        items: Vec<Item>,
        force_trailing_comma: bool,
        surround_with_spaces: bool,
        group: &mut ChunkGroup,
        mut format_item: F,
    ) where
        F: FnMut(&mut Self, Item, &mut ChunkGroup),
    {
        let mut comments_chunk = self.skip_comments_and_whitespace_chunk();

        // Handle leading block vs. line comments a bit differently.
        if comments_chunk.string.trim().starts_with("/*") {
            group.increase_indentation();
            if surround_with_spaces {
                group.space_or_line();
            } else {
                group.line();
            }

            // Note: there's no space before `{}` because it was just produced
            comments_chunk.string = if surround_with_spaces {
                comments_chunk.string.trim().to_string()
            } else {
                format!("{} ", comments_chunk.string.trim())
            };
            group.leading_comment(comments_chunk);
        } else {
            group.increase_indentation();
            if surround_with_spaces {
                group.space_or_line();
            } else {
                group.line();
            }

            group.trailing_comment(comments_chunk);
        }

        for (index, expr) in items.into_iter().enumerate() {
            if index > 0 {
                group.text_attached_to_last_group(self.chunk(|formatter| {
                    formatter.write_comma();
                }));
                let newlines_count_before_comment = self.following_newlines_count();
                group.text(self.chunk(|formatter| {
                    formatter.skip_whitespace();
                }));
                if let Token::BlockComment(..) = &self.token {
                    // We let block comments be part of the item that's going to be formatted
                } else {
                    // Line comments can be trailing or leading, depending on whether there are newlines before them
                    let comments_and_whitespace_chunk = self.skip_comments_and_whitespace_chunk();
                    if !comments_and_whitespace_chunk.string.trim().is_empty() {
                        if newlines_count_before_comment > 0 {
                            group.line();
                            group.leading_comment(comments_and_whitespace_chunk);
                        } else {
                            group.trailing_comment(comments_and_whitespace_chunk);
                        }
                    }
                }
                group.space_or_line();
            }
            format_item(self, expr, group);
        }

        let chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace();

            // Trailing comma
            if formatter.is_at(Token::Comma) {
                formatter.bump();
                formatter.skip_comments_and_whitespace();
            }
        });

        // Make sure to put a trailing comma before the last parameter comments, if there were any
        if !force_trailing_comma {
            group.trailing_comma();
        }

        group.text(chunk);

        if force_trailing_comma {
            group.text(TextChunk::new(",".to_string()));
        }

        group.decrease_indentation();
        if surround_with_spaces {
            group.space_or_line();
        } else {
            group.line();
        }
    }

    fn format_constructor(&mut self, constructor: ConstructorExpression) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
            formatter.format_type(constructor.typ);
            formatter.write_space();
            formatter.write_left_brace();
        }));

        if constructor.fields.is_empty() {
            if let Some(inner_group) = self.empty_block_contents_chunk() {
                group.group(inner_group);
            }
        } else {
            self.format_items_separated_by_comma(
                constructor.fields,
                false, // force trailing comma
                true,  // surround with spaces
                &mut group,
                |formatter, (name, value), chunks| {
                    chunks.text(formatter.chunk(|formatter| {
                        formatter.write_identifier(name);
                        formatter.skip_comments_and_whitespace();
                    }));

                    if formatter.is_at(Token::Colon) {
                        chunks.text(formatter.chunk(|formatter| {
                            formatter.write_token(Token::Colon);
                            formatter.write_space();
                        }));
                        formatter.format_expression(value, chunks);
                    }
                },
            );
        }
        group.text(self.chunk(|formatter| {
            formatter.write_right_brace();
        }));

        group
    }

    fn format_member_access(&mut self, member_access: MemberAccessExpression) -> ChunkGroup {
        let group_tag = self.new_group_tag();

        let mut group = self.format_member_access_impl(
            member_access,
            false, // nested
            group_tag,
        );
        group.force_multiline_on_children_with_same_tag_if_multiline = true;
        group
    }

    fn format_member_access_impl(
        &mut self,
        member_access: MemberAccessExpression,
        nested: bool,
        group_tag: GroupTag,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.tag = Some(group_tag);

        if !nested {
            group.push_indentation();
        }

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

        match member_access.lhs.kind {
            ExpressionKind::MemberAccess(lhs_member_access) => {
                group.group(self.format_member_access_impl(
                    *lhs_member_access,
                    true, // nested
                    group_tag,
                ));
            }
            ExpressionKind::MethodCall(lhs_method_call) => {
                group.group(self.format_method_call_impl(
                    *lhs_method_call,
                    true, // nested
                    group_tag,
                ));
            }
            _ => {
                self.format_expression(member_access.lhs, &mut group);

                increase_indentation = true;
            }
        };

        group.trailing_comment(self.skip_comments_and_whitespace_chunk());

        if increase_indentation {
            group.increase_indentation();
        }

        group.line();

        group.text(self.chunk(|formatter| {
            formatter.write_token(Token::Dot);
            formatter.write_identifier_or_integer(member_access.rhs);
        }));

        if !nested {
            group.pop_indentation();
        }

        group
    }

    fn format_cast(&mut self, cast_expression: CastExpression) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        self.format_expression(cast_expression.lhs, &mut group);
        group.text(self.chunk(|formatter| {
            formatter.write_space();
            formatter.write_keyword(Keyword::As);
            formatter.write_space();
            formatter.format_type(cast_expression.r#type);
        }));
        group
    }

    fn format_prefix(&mut self, prefix: PrefixExpression) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
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
        self.format_expression(prefix.rhs, &mut group);
        group
    }

    fn format_infix_expression(&mut self, infix: InfixExpression) -> ChunkGroup {
        let group_tag = self.new_group_tag();

        let mut group = self.format_infix_expression_with_group_tag(
            infix, group_tag, false, // nested
        );
        group.force_multiline_on_children_with_same_tag_if_multiline = true;
        group
    }

    fn format_infix_expression_with_group_tag(
        &mut self,
        infix: InfixExpression,
        group_tag: GroupTag,
        nested: bool,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.tag = Some(group_tag);

        if !nested {
            group.push_indentation();
        }

        // If we have code like `a + b + c + d`, that's always parsed as `((a + b) + c) + d` where each
        // parentheses denotes an InfixExpression. So, if the lhs of the current infix expression is also
        // an infix expression with the same operator, we format it with the same tag.
        // If the lhs is not an infix expression or has a different operator, we format it normally,
        // and afterwards signal an increase in indentation. That way if this infix expression has
        // to be formatted in multiple lines, we'll only indent after the first operand
        // (we still produce "space or line" after each operator).
        let increase_indentation = match infix.lhs.kind {
            ExpressionKind::Infix(lhs_infix) if lhs_infix.operator == infix.operator => {
                group.group(self.format_infix_expression_with_group_tag(
                    *lhs_infix, group_tag, true, // nested
                ));
                false
            }
            _ => {
                self.format_expression(infix.lhs, &mut group);
                true
            }
        };

        // Indent right after the lhs so that if there's a trailing comment,
        // the next line is indented correctly.
        if increase_indentation {
            group.increase_indentation();
        }

        let mut comment_chunk_after_lhs = self.skip_comments_and_whitespace_chunk();

        // If the comment is not empty but doesn't have newlines, it's surely `/* comment */`.
        // We format that with spaces surrounding it so it looks like `a /* comment */ + b`.
        if !comment_chunk_after_lhs.string.trim().is_empty()
            && !comment_chunk_after_lhs.has_newlines
        {
            // Note: there's no space after `{}` because a bit below comes "space_or_line".
            comment_chunk_after_lhs.string = format!(" {}", comment_chunk_after_lhs.string.trim());
            group.text(comment_chunk_after_lhs);
        } else {
            group.trailing_comment(comment_chunk_after_lhs);
        }

        group.space_or_line();
        group.text(self.chunk(|formatter| {
            let tokens_count =
                if infix.operator.contents == BinaryOpKind::ShiftRight { 2 } else { 1 };
            for _ in 0..tokens_count {
                formatter.write_current_token();
                formatter.bump();
            }
            formatter.write_space();
        }));

        self.format_expression(infix.rhs, &mut group);

        if !nested {
            group.pop_indentation();
        }

        group
    }

    pub(super) fn format_if_expression(
        &mut self,
        if_expression: IfExpression,
        mut force_multiple_lines: bool,
    ) -> ChunkGroup {
        let group_tag = self.new_group_tag();
        let mut group = self.format_if_expression_with_group_tag(
            if_expression,
            &mut force_multiple_lines,
            group_tag,
        );

        if force_multiple_lines || group.width() > self.config.single_line_if_else_max_width {
            force_if_chunks_to_multiple_lines(&mut group, group_tag);
        }

        group
    }

    pub(super) fn format_if_expression_with_group_tag(
        &mut self,
        if_expression: IfExpression,
        force_multiple_lines: &mut bool,
        group_tag: GroupTag,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.tag = Some(group_tag);

        group.text(self.chunk(|formatter| {
            formatter.write_keyword(Keyword::If);
            formatter.write_space();
        }));

        self.format_expression(if_expression.condition, &mut group);

        let comment_chunk_after_condition = self.skip_comments_and_whitespace_chunk();
        if comment_chunk_after_condition.has_newlines {
            *force_multiple_lines = true;
            group.trailing_comment(comment_chunk_after_condition);
        } else {
            group.space(self);
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
        consequence_group.tag = Some(group_tag);
        group.group(consequence_group);

        if let Some(alternative) = if_expression.alternative {
            group.text(self.chunk(|formatter| {
                formatter.write_space();
                formatter.write_keyword(Keyword::Else);
            }));

            let comment_chunk_after_else = self.skip_comments_and_whitespace_chunk();
            if comment_chunk_after_else.has_newlines {
                *force_multiple_lines = true;
                group.trailing_comment(comment_chunk_after_else);
            } else {
                group.space(self);
            }

            let mut alternative_group = match alternative.kind {
                ExpressionKind::Block(block) => {
                    self.format_block_expression(block, *force_multiple_lines)
                }
                ExpressionKind::If(if_expression) => self.format_if_expression_with_group_tag(
                    *if_expression,
                    force_multiple_lines,
                    group_tag,
                ),
                _ => panic!("Unexpected if alternative expression kind"),
            };

            alternative_group.tag = Some(group_tag);
            group.group(alternative_group);
        }

        group
    }

    fn format_index_expression(&mut self, index: IndexExpression) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        self.format_expression(index.collection, &mut group);
        group.text(self.chunk(|formatter| {
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
            group.increase_indentation();
            group.line();
        }

        group.leading_comment(comments_chunk);

        self.format_expression(index.index, &mut group);

        if comments_chunk_has_newlines {
            group.decrease_indentation();
            group.line();
        }

        group.text(self.chunk(|formatter| {
            formatter.write_right_bracket();
        }));
        group
    }

    fn format_call(&mut self, call: CallExpression) -> ChunkGroup {
        let mut group = ChunkGroup::new();

        self.format_expression(*call.func, &mut group);

        group.text(self.chunk(|formatter| {
            if call.is_macro_call {
                formatter.write_token(Token::Bang);
            }
            formatter.write_left_paren();
        }));

        group.kind = GroupKind::ExpressionList {
            prefix_width: group.width(),
            expressions_count: call.arguments.len(),
        };

        // Format arguments in a separate group so we can calculate the arguments
        // width and determine if we need to format this call in multiple lines.
        let mut args_group = ChunkGroup::new();
        self.format_expressions_separated_by_comma(
            call.arguments,
            false, // force trailing comma
            &mut args_group,
        );

        if args_group.width() > self.config.fn_call_width {
            group.force_multiple_lines = true;
        }

        // We no longer need this subgroup, so put all its chunks into the main chunks
        group.chunks.extend(args_group.chunks);

        group.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));

        group
    }

    fn format_method_call(&mut self, method_call: MethodCallExpression) -> ChunkGroup {
        let group_tag = self.new_group_tag();

        let mut group = self.format_method_call_impl(
            method_call,
            false, // nested
            group_tag,
        );
        group.force_multiline_on_children_with_same_tag_if_multiline = true;
        group
    }

    fn format_method_call_impl(
        &mut self,
        method_call: MethodCallExpression,
        nested: bool,
        group_tag: GroupTag,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.tag = Some(group_tag);

        if !nested {
            group.push_indentation();
        }

        // The logic here is similar to that of `format_member_access_with_group_tag`, so
        // please that function inner comments for details.
        let mut increase_indentation_before_dot = false;

        match method_call.object.kind {
            ExpressionKind::MethodCall(lhs_method_call) => {
                group.group(self.format_method_call_impl(
                    *lhs_method_call,
                    true, // nested
                    group_tag,
                ));
            }
            ExpressionKind::MemberAccess(lhs_member_access) => {
                group.group(self.format_member_access_impl(
                    *lhs_member_access,
                    true, // nested
                    group_tag,
                ));
            }
            _ => {
                self.format_expression(method_call.object, &mut group);

                increase_indentation_before_dot = true;
            }
        }

        group.trailing_comment(self.skip_comments_and_whitespace_chunk());

        if increase_indentation_before_dot {
            group.increase_indentation();
        }

        group.line();

        group.text(self.chunk(|formatter| {
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

        group.kind = GroupKind::MethodCall {
            width_until_left_paren_inclusive: group.width(),
            has_newlines_before_left_paren: group.has_newlines(),
            lhs: nested,
        };

        let mut args_group = ChunkGroup::new();
        args_group.kind = GroupKind::ExpressionList {
            prefix_width: 0,
            expressions_count: method_call.arguments.len(),
        };
        self.format_expressions_separated_by_comma(
            method_call.arguments,
            false, // force trailing comma
            &mut args_group,
        );
        group.group(args_group);

        group.text(self.chunk(|formatter| {
            formatter.write_right_paren();
        }));

        if !nested {
            group.pop_indentation();
        }

        group
    }

    pub(super) fn format_block_expression(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
    ) -> ChunkGroup {
        let mut group = ChunkGroup::new();
        group.text(self.chunk(|formatter| {
            formatter.write_left_brace();
        }));
        self.format_block_expression_contents(block, force_multiple_lines, &mut group);
        group.text(self.chunk(|formatter| {
            formatter.write_right_brace();
        }));
        group
    }

    pub(super) fn format_block_expression_contents(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
        group: &mut ChunkGroup,
    ) {
        if block.is_empty() {
            if let Some(block_group) = self.empty_block_contents_chunk() {
                group.chunks.extend(block_group.chunks);
            }
        } else {
            self.format_non_empty_block_expression_contents(block, force_multiple_lines, group);
        }
    }

    pub(super) fn format_non_empty_block_expression_contents(
        &mut self,
        block: BlockExpression,
        force_multiple_lines: bool,
        group: &mut ChunkGroup,
    ) {
        group.force_multiple_lines = force_multiple_lines || block.statements.len() > 1;
        let surround_with_spaces = !group.force_multiple_lines && block.statements.len() == 1;

        group.increase_indentation();
        if surround_with_spaces {
            group.space_or_line();
        } else {
            group.line();
        }

        for (index, statement) in block.statements.into_iter().enumerate() {
            let mut ignore_next = false;

            if index > 0 {
                let count = self.following_newlines_count();
                if count > 0 {
                    // If newlines follow, we first add a line, then add the comment chunk
                    group.lines(count > 1);
                    group.leading_comment(self.chunk(|formatter| {
                        formatter.skip_comments_and_whitespace_writing_multiple_lines_if_found();
                    }));
                    ignore_next = self.ignore_next;
                } else {
                    // Otherwise, add the comment first as it's a trailing comment
                    group.trailing_comment(self.chunk(|formatter| {
                        formatter.skip_comments_and_whitespace_writing_multiple_lines_if_found();
                    }));
                    ignore_next = self.ignore_next;
                    group.line();
                }
            }

            self.format_statement(statement, group, ignore_next);
        }

        // See how many newlines follow the last statement
        let count = self.following_newlines_count();

        group.text(self.chunk(|formatter| {
            formatter.skip_whitespace();
        }));

        // After skipping whitespace we check if there's a comment. If so, we respect
        // how many lines were before that comment.
        if count > 0 && matches!(self.token, Token::LineComment(..) | Token::BlockComment(..)) {
            group.lines(count > 1);
        }

        // Finally format the comment, if any
        group.text(self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace_writing_multiple_lines_if_found();
        }));

        group.decrease_indentation();

        if surround_with_spaces {
            group.space_or_line();
        } else {
            group.line();
        }
    }

    pub(super) fn empty_block_contents_chunk(&mut self) -> Option<ChunkGroup> {
        let mut group = ChunkGroup::new();
        group.increase_indentation();

        let newlines_count = self.following_newlines_count();

        let mut chunk = self.chunk(|formatter| {
            formatter.skip_comments_and_whitespace_writing_multiple_lines_if_found();
        });

        if chunk.string.trim().is_empty() {
            // We only found whitespace until the next non-whitespace-non-comment token,
            // so there's nothing to write.
            None
        } else {
            // If we have a trailing comment, preserve it in the same line
            if newlines_count == 0 && !chunk.string.trim_start().starts_with("//") {
                chunk.string = format!(" {} ", chunk.string.trim());
            }
            group.text(chunk);
            group.decrease_indentation();
            group.line();
            Some(group)
        }
    }
}

impl<'a> Formatter<'a> {
    pub(super) fn format_empty_block_contents(&mut self) {
        if let Some(chunks) = self.chunk_formatter().empty_block_contents_chunk() {
            self.format_chunk_group(chunks);
        }
    }
}

fn force_if_chunks_to_multiple_lines(group: &mut ChunkGroup, group_tag: GroupTag) {
    if group.tag == Some(group_tag) {
        group.force_multiple_lines = true;
    }

    for chunk in group.chunks.iter_mut() {
        if let Chunk::Group(inner_group) = chunk {
            force_if_chunks_to_multiple_lines(inner_group, group_tag);
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
        let expected = "global x =
    [1, 2, 3, 4, 5];
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
        let expected = "global x =
    [1, 2, 3, 4, 5];

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
    fn format_long_array_element() {
        let src = "global x = [ 123, 1234, 12345, 123, 1234, 12345, 123456, 123] ;";
        let expected = "global x = [
    123,
    1234,
    12345,
    123,
    1234,
    12345,
    123456,
    123,
];
";

        let config =
            Config { short_array_element_width_threshold: 5, max_width: 30, ..Default::default() };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_short_array_with_block_comment_before_elements() {
        let src = "global x = [ /* one */ 1, /* two */ 2 ] ;";
        let expected = "global x = [/* one */ 1, /* two */ 2];\n";

        assert_format(src, expected);
    }

    #[test]
    fn format_long_array_with_block_comment_before_elements() {
        let src = "global x = [ /* one */ 1, /* two */ 123456789012345, 3, 4 ] ;";
        let expected = "global x = [
    /* one */ 1,
    /* two */ 123456789012345,
    3,
    4,
];
";

        let config =
            Config { short_array_element_width_threshold: 5, max_width: 30, ..Config::default() };
        assert_format_with_config(src, expected, config);
    }

    #[test]
    fn format_long_array_with_line_comment_before_elements() {
        let src = "global x = [
    // one
    1,
    // two
    123456789012345,
    3,
    4,
];
";
        let expected = "global x = [
    // one
    1,
    // two
    123456789012345,
    3,
    4,
];
";

        let config =
            Config { short_array_element_width_threshold: 5, max_width: 30, ..Config::default() };
        assert_format_with_config(src, expected, config);
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
    fn format_infix_with_trailing_comments() {
        let src = "fn foo() {
    let x = 1 // one
+ 2 // two
+ 3; // three
}
";
        let expected = "fn foo() {
    let x = 1 // one
        + 2 // two
        + 3; // three
}
";
        assert_format(src, expected);
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
    fn format_nested_call_with_maximum_width() {
        let src = "fn foo() { foo(bar(123, 456, 789)) } ";
        let expected = "fn foo() {
    foo(bar(
        123,
        456,
        789,
    ))
}
";
        assert_format_with_max_width(src, expected, "    foo(bar(".len());
    }

    #[test]
    fn format_nested_call_with_maximum_width_2() {
        let src = "fn foo() {
    let note_interface_impl = s.as_type().get_trait_impl(quote { crate::note::note_interface::NoteInterface<$serialized_len_type> }
        .as_trait_constraint());
}
";
        let expected = "fn foo() {
    let note_interface_impl = s.as_type().get_trait_impl(
        quote { crate::note::note_interface::NoteInterface<$serialized_len_type> }
            .as_trait_constraint(),
    );
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_nested_call_with_maximum_width_3() {
        let src = "mod foo {
    fn bar() {
        assert(foo(bar.baz(x12345)));
    }
}
";
        let expected = "mod foo {
    fn bar() {
        assert(foo(bar.baz(
            x12345,
        )));
    }
}
";
        assert_format_with_max_width(src, expected, 33);
    }

    #[test]
    fn format_nested_call_with_maximum_width_4() {
        let src = "mod foo {
    fn bar() {
        assert(foo(bar_baz(x1, x2)));
    }
}
";
        let expected = "mod foo {
    fn bar() {
        assert(foo(bar_baz(
            x1,
            x2,
        )));
    }
}
";
        assert_format_with_max_width(src, expected, 33);
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
        let expected = "global x = foo::bar(|x, y| {
    some_chunk_of_code
});
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
        let expected = "global x = bar
    .baz(1, 2)
    .qux(1, 2, 3)
    .one(5, 6);
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_method_call_chain_2() {
        let src = "fn foo() { bar . baz ( 1, 2 ) . qux ( 1 , 2, 3) . one ( 5, 6)   }";
        let expected = "fn foo() {
    bar.baz(1, 2).qux(1, 2, 3).one(
        5,
        6,
    )
}
";
        assert_format_with_max_width(src, expected, "    bar.baz(1, 2).qux(1, 2, 3).one(".len());
    }

    #[test]
    fn format_method_call_chain_3() {
        let src = "fn foo() {     assert(p4_affine.eq(Gaffine::new(6890855772600357754907169075114257697580319025794532037257385534741338397365, 4338620300185947561074059802482547481416142213883829469920100239455078257889)));  }";
        let expected = "fn foo() {
    assert(p4_affine.eq(Gaffine::new(
        6890855772600357754907169075114257697580319025794532037257385534741338397365,
        4338620300185947561074059802482547481416142213883829469920100239455078257889,
    )));
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_method_call_with_maximum_width() {
        let src = "global x =  foo::bar.baz( );";
        let expected = "global x = foo::bar
    .baz();
";
        assert_format_with_max_width(src, expected, "foo::bar.baz".len() - 1);
    }

    #[test]
    fn format_nested_method_call_with_maximum_width() {
        let src = "fn foo() { foo.bar(baz.qux(123, 456, 789)) } ";
        let expected = "fn foo() {
    foo.bar(baz.qux(
        123,
        456,
        789,
    ))
}
";
        assert_format_with_max_width(src, expected, "    foo.bar(bar.qux(".len());
    }

    #[test]
    fn format_nested_method_call_with_maximum_width_2() {
        let src = "fn foo() {
    assert(
        p4_affine.eq(Gaffine::new(
            6890855772600357754907169075114257697580319025794532037257385534741338397365,
            4338620300185947561074059802482547481416142213883829469920100239455078257889,
        )),
    );
}
";
        let expected = "fn foo() {
    assert(p4_affine.eq(Gaffine::new(
        6890855772600357754907169075114257697580319025794532037257385534741338397365,
        4338620300185947561074059802482547481416142213883829469920100239455078257889,
    )));
}
";
        assert_format(src, expected);
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
        let expected = "global x = foo
    .bar
    .baz
    .qux
    .final;
";
        assert_format_with_max_width(src, expected, "foo.bar.baz.qux.final".len() - 1);
    }

    #[test]
    fn format_long_member_access_and_method_call_chain() {
        let src = "global x =  foo . bar(1, 2) . baz . qux(2, 3) . this_is_a_long_name   ;";
        let expected = "global x = foo
    .bar(1, 2)
    .baz
    .qux(2, 3)
    .this_is_a_long_name;
";
        assert_format_with_max_width(src, expected, 25);
    }

    #[test]
    fn format_long_member_access_and_method_call_chain_2() {
        let src = "fn burn() {
    storage
        .at(from)
        .sub(from_keys.npk_m, U128::from_integer(amount))
        .emit(encode_and_encrypt_note!());
}
";
        let expected = "fn burn() {
    storage.at(from).sub(from_keys.npk_m, U128::from_integer(amount)).emit(
        encode_and_encrypt_note!(),
    );
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
        let src = "global x = unsafe { 
        1  } ;";
        let expected = "global x = unsafe { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_unsafe_two_expressions() {
        let src = "global x = unsafe { 
        1; 2  } ;";
        let expected = "global x = unsafe {
    1;
    2
};
";
        assert_format(src, expected);
    }

    #[test]
    fn format_unsafe_with_doc_comment() {
        let src = "fn foo() {
        /// Comment 
        unsafe { 1  } }";
        let expected = "fn foo() {
    /// Comment
    unsafe {
        1
    }
}
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
    fn format_quote_with_newlines() {
        let src = "fn foo() {
    quote {

        foo

        bar

    }
}
";
        let expected = src;
        assert_format(src, expected);
    }

    #[test]
    fn format_quote_with_bracket_delimiter() {
        let src = "global x = quote [ 1  2  3 $four $(five) ];";
        let expected = "global x = quote [ 1  2  3 $four $(five) ];\n";
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
    fn format_lambda_with_block_simplifies() {
        let src = "global x = | |  {  1  } ;";
        let expected = "global x = || 1;\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_block_does_not_simplify_if_it_ends_with_semicolon() {
        let src = "global x = | |  {  1;  } ;";
        let expected = "global x = || { 1; };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_block_does_not_simplify_if_it_has_return_type() {
        let src = "global x = | | -> i32  {  1  } ;";
        let expected = "global x = || -> i32 { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_simplifies_block_with_quote() {
        let src = "global x = | | {  quote { 1 }   } ;";
        let expected = "global x = || quote { 1 };\n";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_with_block_simplifies_inside_arguments_list() {
        let src = "global x = some_call(this_is_a_long_argument, | |  {  1  });";
        let expected = "global x = some_call(
    this_is_a_long_argument,
    || 1,
);
";
        assert_format_with_max_width(src, expected, 20);
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
    fn format_lambda_as_last_method_call_argument_2() {
        let src = "fn foo(){
    m.structs().any(|s: StructDefinition| s.has_named_attribute(\"storage\") | s.has_named_attribute(\"storage_no_init\"),
    )
}
";
        let expected = "fn foo() {
    m.structs().any(|s: StructDefinition| {
        s.has_named_attribute(\"storage\") | s.has_named_attribute(\"storage_no_init\")
    })
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_method_call_argument_3() {
        let src = "mod moo {
    fn foo() {
        let mut sorted_write_tuples = unsafe {
            get_sorted_tuple(
                final_public_data_writes.storage,
                |(_, leaf_a): (u32, PublicDataTreeLeaf), (_, leaf_b): (u32, PublicDataTreeLeaf)| full_field_less_than(
                    1, 2,
                ),
            )
        };
    }
}
";
        let expected = "mod moo {
    fn foo() {
        let mut sorted_write_tuples = unsafe {
            get_sorted_tuple(
                final_public_data_writes.storage,
                |(_, leaf_a): (u32, PublicDataTreeLeaf), (_, leaf_b): (u32, PublicDataTreeLeaf)| {
                    full_field_less_than(1, 2)
                },
            )
        };
    }
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_method_call_chain_argument() {
        let src = "global x = foo.bar(1).baz(2, |x| { 1; 2 });";
        let expected = "global x = foo.bar(1).baz(2, |x| {
    1;
    2
});
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_method_call_chain_argument_2() {
        let src = "fn foo() { expr.as_unsafe().map(|exprs| { a; aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa }) }
        ";
        let expected = "fn foo() {
    expr.as_unsafe().map(|exprs| {
        a;
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    })
}
";
        assert_format(src, expected);
    }

    #[test]
    fn format_lambda_as_last_method_call_has_to_wrap() {
        let src = "global foo = bar(1, 2, 3, |argument| { 1; 2 });";
        let expected = "global foo = bar(
    1,
    2,
    3,
    |argument| {
        1;
        2
    },
);
";
        assert_format_with_max_width(src, expected, src.len() - 10);
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

    #[test]
    fn attaches_comma_to_last_group() {
        let src = " mod moo {
    fn foo() {
        [
            Foo { a: 1 }, Foo { a: 1 }
            ];
            bar;
        }
}
";
        let expected = "mod moo {
    fn foo() {
        [
            Foo {
                a: 1,
            },
            Foo {
                a: 1,
            },
        ];
        bar;
    }
}
";
        assert_format_with_max_width(src, expected, "            Foo { a: 1 },".len() - 1);
    }
}
