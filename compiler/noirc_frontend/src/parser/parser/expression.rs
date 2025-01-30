use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, CastExpression, ConstructorExpression,
        Expression, ExpressionKind, Ident, IfExpression, IndexExpression, Literal,
        MemberAccessExpression, MethodCallExpression, Statement, TypePath, UnaryOp, UnresolvedType,
    },
    parser::{labels::ParsingRuleLabel, parser::parse_many::separated_by_comma, ParserErrorReason},
    token::{Keyword, Token, TokenKind},
};

use super::{
    parse_many::{
        separated_by_comma_until_right_brace, separated_by_comma_until_right_paren,
        without_separator,
    },
    Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression_or_error(&mut self) -> Expression {
        self.parse_expression_or_error_impl(true) // allow constructors
    }

    /// Expression = EqualOrNotEqualExpression
    pub(crate) fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_expression_impl(true) // allow constructors
    }

    /// When parsing `if` conditions we don't allow constructors.
    /// For example `if foo { 1 }` shouldn't have `foo { 1 }` as the condition, but `foo` instead.
    /// The same goes with `for`: `for x in foo { 1 }` should have `foo` as the collection, not `foo { 1 }`.
    ///
    /// ExpressionExceptConstructor = "Expression except ConstructorException"
    pub(crate) fn parse_expression_except_constructor_or_error(&mut self) -> Expression {
        self.parse_expression_or_error_impl(false) // allow constructors
    }

    pub(crate) fn parse_expression_or_error_impl(
        &mut self,
        allow_constructors: bool,
    ) -> Expression {
        if let Some(expr) = self.parse_expression_impl(allow_constructors) {
            expr
        } else {
            self.push_expected_expression();
            Expression { kind: ExpressionKind::Error, span: self.span_at_previous_token_end() }
        }
    }

    fn parse_expression_impl(&mut self, allow_constructors: bool) -> Option<Expression> {
        self.parse_equal_or_not_equal(allow_constructors)
    }

    /// Term
    ///    = UnaryOp Term
    ///    | AtomOrUnaryRightExpression
    pub(super) fn parse_term(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;

        if let Some(operator) = self.parse_unary_op() {
            let Some(rhs) = self.parse_term(allow_constructors) else {
                self.expected_label(ParsingRuleLabel::Expression);
                return None;
            };
            let kind = ExpressionKind::prefix(operator, rhs);
            let span = self.span_since(start_span);
            return Some(Expression { kind, span });
        }

        self.parse_atom_or_unary_right(allow_constructors)
    }

    /// UnaryOp = '&' 'mut' | '-' | '!' | '*'
    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        if self.at(Token::Ampersand) && self.next_is(Token::Keyword(Keyword::Mut)) {
            self.bump();
            self.bump();
            Some(UnaryOp::MutableReference)
        } else if self.eat(Token::Minus) {
            Some(UnaryOp::Minus)
        } else if self.eat(Token::Bang) {
            Some(UnaryOp::Not)
        } else if self.eat(Token::Star) {
            Some(UnaryOp::Dereference { implicitly_added: false })
        } else {
            None
        }
    }

    /// AtomOrUnaryRightExpression
    ///     = Atom
    ///     | UnaryRightExpression
    fn parse_atom_or_unary_right(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let mut atom = self.parse_atom(allow_constructors)?;
        let mut parsed;

        loop {
            (atom, parsed) = self.parse_unary_right(atom, start_span);
            if parsed {
                continue;
            } else {
                break;
            }
        }

        Some(atom)
    }

    /// UnaryRightExpression
    ///     = CallExpression
    ///     | MemberAccessOrMethodCallExpression
    ///     | CastExpression
    ///     | IndexExpression
    fn parse_unary_right(&mut self, mut atom: Expression, start_span: Span) -> (Expression, bool) {
        let mut parsed;

        (atom, parsed) = self.parse_call(atom, start_span);
        if parsed {
            return (atom, parsed);
        }

        (atom, parsed) = self.parse_member_access_or_method_call(atom, start_span);
        if parsed {
            return (atom, parsed);
        }

        (atom, parsed) = self.parse_cast(atom, start_span);
        if parsed {
            return (atom, parsed);
        }

        self.parse_index(atom, start_span)
    }

    /// CallExpression = Atom CallArguments
    fn parse_call(&mut self, atom: Expression, start_span: Span) -> (Expression, bool) {
        if let Some(call_arguments) = self.parse_call_arguments() {
            let kind = ExpressionKind::Call(Box::new(CallExpression {
                func: Box::new(atom),
                arguments: call_arguments.arguments,
                is_macro_call: call_arguments.is_macro_call,
            }));
            let span = self.span_since(start_span);
            let atom = Expression { kind, span };
            (atom, true)
        } else {
            (atom, false)
        }
    }

    /// MemberAccessOrMethodCallExpression
    ///     = MemberAccessExpression
    ///     | MethodCallExpression
    ///
    /// MemberAccessExpression = Atom '.' identifier
    ///
    /// MethodCallExpression = Atom '.' identifier CallArguments
    fn parse_member_access_or_method_call(
        &mut self,
        atom: Expression,
        start_span: Span,
    ) -> (Expression, bool) {
        if !self.eat_dot() {
            return (atom, false);
        }

        let Some(field_name) = self.parse_member_access_field_name() else { return (atom, true) };

        let generics = self.parse_generics_after_member_access_field_name();

        let kind = if let Some(call_arguments) = self.parse_call_arguments() {
            ExpressionKind::MethodCall(Box::new(MethodCallExpression {
                object: atom,
                method_name: field_name,
                generics,
                arguments: call_arguments.arguments,
                is_macro_call: call_arguments.is_macro_call,
            }))
        } else {
            ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
                lhs: atom,
                rhs: field_name,
            }))
        };

        let span = self.span_since(start_span);
        let atom = Expression { kind, span };
        (atom, true)
    }

    fn parse_member_access_field_name(&mut self) -> Option<Ident> {
        if let Some(ident) = self.eat_ident() {
            Some(ident)
        } else if let Some(int) = self.eat_int() {
            Some(Ident::new(int.to_string(), self.previous_token_span))
        } else {
            self.push_error(
                ParserErrorReason::ExpectedFieldName(self.token.token().clone()),
                self.current_token_span,
            );
            None
        }
    }

    /// CastExpression = Atom 'as' Type
    fn parse_cast(&mut self, atom: Expression, start_span: Span) -> (Expression, bool) {
        if !self.eat_keyword(Keyword::As) {
            return (atom, false);
        }

        let typ = self.parse_type_or_error();
        let kind = ExpressionKind::Cast(Box::new(CastExpression { lhs: atom, r#type: typ }));
        let span = self.span_since(start_span);
        let atom = Expression { kind, span };
        (atom, true)
    }

    /// IndexExpression = Atom '[' Expression ']'
    fn parse_index(&mut self, atom: Expression, start_span: Span) -> (Expression, bool) {
        if !self.eat_left_bracket() {
            return (atom, false);
        }

        let index = self.parse_expression_or_error();
        self.eat_or_error(Token::RightBracket);
        let kind = ExpressionKind::Index(Box::new(IndexExpression { collection: atom, index }));
        let span = self.span_since(start_span);
        let atom = Expression { kind, span };
        (atom, true)
    }

    fn parse_generics_after_member_access_field_name(&mut self) -> Option<Vec<UnresolvedType>> {
        if self.eat_double_colon() {
            let generics =
                self.parse_path_generics(ParserErrorReason::AssociatedTypesNotAllowedInMethodCalls);
            if generics.is_none() {
                self.expected_token(Token::Less);
            }
            generics
        } else {
            None
        }
    }

    /// Atom
    ///     = Literal
    ///     | ParenthesesExpression
    ///     | UnsafeExpression
    ///     | PathExpression
    ///     | IfExpression
    ///     | Lambda
    ///     | ComptimeExpression
    ///     | UnquoteExpression
    ///     | TypePathExpression
    ///     | AsTraitPath
    ///     | ResolvedExpression
    ///     | InternedExpression
    ///     | InternedStatementExpression
    fn parse_atom(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_span = self.current_token_span;
        let kind = self.parse_atom_kind(allow_constructors)?;
        Some(Expression { kind, span: self.span_since(start_span) })
    }

    fn parse_atom_kind(&mut self, allow_constructors: bool) -> Option<ExpressionKind> {
        let span_before_doc_comments = self.current_token_span;
        let doc_comments = self.parse_outer_doc_comments();
        let has_doc_comments = !doc_comments.is_empty();

        if let Some(kind) = self.parse_unsafe_expr(&doc_comments, span_before_doc_comments) {
            return Some(kind);
        }

        if has_doc_comments {
            self.push_error(
                ParserErrorReason::DocCommentDoesNotDocumentAnything,
                self.span_since(span_before_doc_comments),
            );
        }

        if let Some(literal) = self.parse_literal() {
            return Some(literal);
        }

        if let Some(kind) = self.parse_parentheses_expression() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_path_expr(allow_constructors) {
            return Some(kind);
        }

        // A constructor where the type is an interned unresolved type data is valid
        if matches!(self.token.token(), Token::InternedUnresolvedTypeData(..))
            && self.next_is(Token::LeftBrace)
        {
            let span = self.current_token_span;
            let typ = self.parse_interned_type().unwrap();
            self.eat_or_error(Token::LeftBrace);
            let typ = UnresolvedType { typ, span };
            return Some(self.parse_constructor(typ));
        }

        if let Some(kind) = self.parse_if_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_lambda() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_comptime_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_unquote_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_type_path_expr() {
            return Some(kind);
        }

        if let Some(as_trait_path) = self.parse_as_trait_path() {
            return Some(ExpressionKind::AsTraitPath(as_trait_path));
        }

        if let Some(kind) = self.parse_resolved_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_interned_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_interned_statement_expr() {
            return Some(kind);
        }

        None
    }

    /// ResolvedExpression = unquote_marker
    fn parse_resolved_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::UnquoteMarker) {
            match token.into_token() {
                Token::UnquoteMarker(expr_id) => return Some(ExpressionKind::Resolved(expr_id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    /// InternedExpression = interned_expr
    fn parse_interned_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::InternedExpr) {
            match token.into_token() {
                Token::InternedExpr(id) => return Some(ExpressionKind::Interned(id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    /// InternedStatementExpression = interned_statement
    fn parse_interned_statement_expr(&mut self) -> Option<ExpressionKind> {
        if let Some(token) = self.eat_kind(TokenKind::InternedStatement) {
            match token.into_token() {
                Token::InternedStatement(id) => return Some(ExpressionKind::InternedStatement(id)),
                _ => unreachable!(""),
            }
        }

        None
    }

    /// UnsafeExpression = 'unsafe' Block
    fn parse_unsafe_expr(
        &mut self,
        doc_comments: &[String],
        span_before_doc_comments: Span,
    ) -> Option<ExpressionKind> {
        let start_span = self.current_token_span;

        if !self.eat_keyword(Keyword::Unsafe) {
            return None;
        }

        if doc_comments.is_empty() {
            if let Some(statement_doc_comments) = &mut self.statement_doc_comments {
                statement_doc_comments.read = true;

                let doc_comments = &statement_doc_comments.doc_comments;
                let span_before_doc_comments = statement_doc_comments.start_span;
                let span_after_doc_comments = statement_doc_comments.end_span;

                if !doc_comments[0].trim().to_lowercase().starts_with("safety:") {
                    self.push_error(
                        ParserErrorReason::UnsafeDocCommentDoesNotStartWithSafety,
                        Span::from(
                            span_before_doc_comments.start()..span_after_doc_comments.start(),
                        ),
                    );
                }
            } else {
                self.push_error(ParserErrorReason::MissingSafetyComment, start_span);
            }
        } else if !doc_comments[0].trim().to_lowercase().starts_with("safety:") {
            self.push_error(
                ParserErrorReason::UnsafeDocCommentDoesNotStartWithSafety,
                self.span_since(span_before_doc_comments),
            );
        }

        if let Some(block) = self.parse_block() {
            Some(ExpressionKind::Unsafe(block, self.span_since(start_span)))
        } else {
            Some(ExpressionKind::Error)
        }
    }

    /// PathExpression
    ///     = VariableExpression
    ///     | ConstructorExpression
    ///
    /// VariableExpression = Path
    fn parse_path_expr(&mut self, allow_constructors: bool) -> Option<ExpressionKind> {
        let path = self.parse_path()?;

        if allow_constructors && self.eat_left_brace() {
            let typ = UnresolvedType::from_path(path);
            return Some(self.parse_constructor(typ));
        }

        Some(ExpressionKind::Variable(path))
    }

    /// ConstructorExpression = Type '{' ConstructorFields? '}'
    ///
    /// ConstructorFields = ConstructorField ( ',' ConstructorField )* ','?
    ///
    /// ConstructorField = identifier ( ':' Expression )?
    fn parse_constructor(&mut self, typ: UnresolvedType) -> ExpressionKind {
        let fields = self.parse_many(
            "constructor fields",
            separated_by_comma_until_right_brace(),
            Self::parse_constructor_field,
        );

        ExpressionKind::Constructor(Box::new(ConstructorExpression {
            typ,
            fields,
            struct_type: None,
        }))
    }

    fn parse_constructor_field(&mut self) -> Option<(Ident, Expression)> {
        let ident = self.eat_ident()?;

        Some(if self.eat_colon() {
            let expression = self.parse_expression_or_error();
            (ident, expression)
        } else if self.at(Token::DoubleColon) || self.at(Token::Assign) {
            // If we find '='  or '::' instead of ':', assume the user meant ':`, error and continue
            self.expected_token(Token::Colon);
            self.bump();
            let expression = self.parse_expression_or_error();
            (ident, expression)
        } else {
            (ident.clone(), ident.into())
        })
    }

    /// IfExpression = 'if' ExpressionExceptConstructor Block ( 'else' ( Block | IfExpression ) )?
    pub(super) fn parse_if_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::If) {
            return None;
        }

        let condition = self.parse_expression_except_constructor_or_error();

        let start_span = self.current_token_span;
        let Some(consequence) = self.parse_block() else {
            self.expected_token(Token::LeftBrace);
            let span = self.span_at_previous_token_end();
            return Some(ExpressionKind::If(Box::new(IfExpression {
                condition,
                consequence: Expression { kind: ExpressionKind::Error, span },
                alternative: None,
            })));
        };
        let span = self.span_since(start_span);
        let consequence = Expression { kind: ExpressionKind::Block(consequence), span };

        let alternative = if self.eat_keyword(Keyword::Else) {
            let start_span = self.current_token_span;
            if let Some(block) = self.parse_block() {
                let span = self.span_since(start_span);
                Some(Expression { kind: ExpressionKind::Block(block), span })
            } else if let Some(if_expr) = self.parse_if_expr() {
                Some(Expression { kind: if_expr, span: self.span_since(start_span) })
            } else {
                self.expected_token(Token::LeftBrace);
                None
            }
        } else {
            None
        };

        Some(ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative })))
    }

    /// ComptimeExpression = 'comptime' Block
    fn parse_comptime_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::Comptime) {
            return None;
        }

        let start_span = self.current_token_span;

        let Some(block) = self.parse_block() else {
            self.expected_token(Token::LeftBrace);
            return None;
        };

        Some(ExpressionKind::Comptime(block, self.span_since(start_span)))
    }

    /// UnquoteExpression
    ///     = '$' identifier
    ///     | '$' '(' Expression ')'
    fn parse_unquote_expr(&mut self) -> Option<ExpressionKind> {
        let start_span = self.current_token_span;

        if !self.eat(Token::DollarSign) {
            return None;
        }

        if let Some(path) = self.parse_path() {
            let expr = Expression {
                kind: ExpressionKind::Variable(path),
                span: self.span_since(start_span),
            };
            return Some(ExpressionKind::Unquote(Box::new(expr)));
        }

        let span_at_left_paren = self.current_token_span;
        if self.eat_left_paren() {
            let expr = self.parse_expression_or_error();
            self.eat_or_error(Token::RightParen);
            let expr = Expression {
                kind: ExpressionKind::Parenthesized(Box::new(expr)),
                span: self.span_since(span_at_left_paren),
            };
            return Some(ExpressionKind::Unquote(Box::new(expr)));
        }

        self.push_error(
            ParserErrorReason::ExpectedIdentifierOrLeftParenAfterDollar,
            self.current_token_span,
        );

        None
    }

    /// TypePathExpression = PrimitiveType '::' identifier ( '::' GenericTypeArgs )?
    fn parse_type_path_expr(&mut self) -> Option<ExpressionKind> {
        let start_span = self.current_token_span;
        let typ = self.parse_primitive_type()?;
        let typ = UnresolvedType { typ, span: self.span_since(start_span) };

        self.eat_or_error(Token::DoubleColon);

        let item = if let Some(ident) = self.eat_ident() {
            ident
        } else {
            self.expected_identifier();
            Ident::new(String::new(), self.span_at_previous_token_end())
        };

        let turbofish = self.eat_double_colon().then(|| {
            let generics = self.parse_generic_type_args();
            if generics.is_empty() {
                self.expected_token(Token::Less);
            }
            generics
        });

        Some(ExpressionKind::TypePath(TypePath { typ, item, turbofish }))
    }

    /// Literal
    ///     = bool
    ///     | int
    ///     | str
    ///     | rawstr
    ///     | fmtstr
    ///     | QuoteExpression
    ///     | ArrayExpression
    ///     | SliceExpression
    ///     | BlockExpression
    ///
    /// QuoteExpression = 'quote' '{' token* '}'
    ///
    /// ArrayExpression = ArrayLiteral
    ///
    /// BlockExpression = Block
    fn parse_literal(&mut self) -> Option<ExpressionKind> {
        if let Some(bool) = self.eat_bool() {
            return Some(ExpressionKind::boolean(bool));
        }

        if let Some(int) = self.eat_int() {
            return Some(ExpressionKind::integer(int));
        }

        if let Some(string) = self.eat_str() {
            return Some(ExpressionKind::string(string));
        }

        if let Some((string, n)) = self.eat_raw_str() {
            return Some(ExpressionKind::raw_string(string, n));
        }

        if let Some((fragments, length)) = self.eat_fmt_str() {
            return Some(ExpressionKind::format_string(fragments, length));
        }

        if let Some(tokens) = self.eat_quote() {
            return Some(ExpressionKind::Quote(tokens));
        }

        if let Some(literal) = self.parse_array_literal() {
            return Some(ExpressionKind::Literal(Literal::Array(literal)));
        }

        if let Some(literal) = self.parse_slice_literal() {
            return Some(ExpressionKind::Literal(Literal::Slice(literal)));
        }

        if let Some(kind) = self.parse_block() {
            return Some(ExpressionKind::Block(kind));
        }

        None
    }

    /// ArrayLiteral
    ///     = StandardArrayLiteral
    ///     | RepeatedArrayLiteral
    ///
    /// StandardArrayLiteral = '[' ArrayElements? ']'
    ///
    /// ArrayElements = Expression ( ',' Expression )? ','?
    ///
    /// RepeatedArrayLiteral = '[' Expression ';' TypeExpression ']'
    fn parse_array_literal(&mut self) -> Option<ArrayLiteral> {
        if !self.eat_left_bracket() {
            return None;
        }

        if self.eat_right_bracket() {
            return Some(ArrayLiteral::Standard(Vec::new()));
        }

        let first_expr = self.parse_expression_or_error();
        if first_expr.kind == ExpressionKind::Error {
            return Some(ArrayLiteral::Standard(Vec::new()));
        }

        if self.eat_semicolon() {
            let length = self.parse_expression_or_error();
            self.eat_or_error(Token::RightBracket);
            return Some(ArrayLiteral::Repeated {
                repeated_element: Box::new(first_expr),
                length: Box::new(length),
            });
        }

        let comma_after_first_expr = self.eat_comma();
        let second_expr_span = self.current_token_span;

        let mut exprs = self.parse_many(
            "expressions",
            separated_by_comma().until(Token::RightBracket),
            Self::parse_expression_in_list,
        );

        if !exprs.is_empty() && !comma_after_first_expr {
            self.expected_token_separating_items(Token::Comma, "expressions", second_expr_span);
        }

        exprs.insert(0, first_expr);

        Some(ArrayLiteral::Standard(exprs))
    }

    /// SliceExpression = '&' ArrayLiteral
    fn parse_slice_literal(&mut self) -> Option<ArrayLiteral> {
        if !(self.at(Token::Ampersand) && self.next_is(Token::LeftBracket)) {
            return None;
        }

        self.bump();
        self.parse_array_literal()
    }

    /// ParenthesesExpression
    ///     = UnitLiteral
    ///     | ParenthesizedExpression
    ///     | TupleExpression
    ///
    /// UnitLiteral = '(' ')'
    ///
    /// ParenthesizedExpression = '(' Expression ')'
    ///
    /// TupleExpression = '(' Expression ( ',' Expression )+ ','? ')'
    fn parse_parentheses_expression(&mut self) -> Option<ExpressionKind> {
        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            return Some(ExpressionKind::Literal(Literal::Unit));
        }

        let (mut exprs, trailing_comma) = self.parse_many_return_trailing_separator_if_any(
            "expressions",
            separated_by_comma_until_right_paren(),
            Self::parse_expression_in_list,
        );

        Some(if exprs.len() == 1 && !trailing_comma {
            ExpressionKind::Parenthesized(Box::new(exprs.remove(0)))
        } else {
            ExpressionKind::Tuple(exprs)
        })
    }

    pub(super) fn parse_expression_in_list(&mut self) -> Option<Expression> {
        if let Some(expr) = self.parse_expression() {
            Some(expr)
        } else {
            self.expected_label(ParsingRuleLabel::Expression);
            None
        }
    }

    /// Block = '{' Statement* '}'
    pub(super) fn parse_block(&mut self) -> Option<BlockExpression> {
        if !self.eat_left_brace() {
            return None;
        }

        let statements = self.parse_many(
            "statements",
            without_separator().until(Token::RightBrace),
            Self::parse_statement_in_block,
        );

        let statements = self.check_statements_require_semicolon(statements);

        Some(BlockExpression { statements })
    }

    fn parse_statement_in_block(&mut self) -> Option<(Statement, (Option<Token>, Span))> {
        if let Some(statement) = self.parse_statement() {
            Some(statement)
        } else {
            self.expected_label(ParsingRuleLabel::Statement);
            None
        }
    }

    fn check_statements_require_semicolon(
        &mut self,
        statements: Vec<(Statement, (Option<Token>, Span))>,
    ) -> Vec<Statement> {
        let last = statements.len().saturating_sub(1);
        let iter = statements.into_iter().enumerate();
        vecmap(iter, |(i, (statement, (semicolon, span)))| {
            statement
                .add_semicolon(semicolon, span, i == last, &mut |error| self.errors.push(error))
        })
    }

    pub(super) fn push_expected_expression(&mut self) {
        self.expected_label(ParsingRuleLabel::Expression);
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        ast::{
            ArrayLiteral, BinaryOpKind, Expression, ExpressionKind, Literal, StatementKind,
            UnaryOp, UnresolvedTypeData,
        },
        parser::{
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
            Parser, ParserErrorReason,
        },
        token::Token,
    };

    fn parse_expression_no_errors(src: &str) -> Expression {
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.span.end() as usize, src.len());
        expect_no_errors(&parser.errors);
        expr
    }

    #[test]
    fn parses_bool_literals() {
        let src = "true";
        let expr = parse_expression_no_errors(src);
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(true))));

        let src = "false";
        let expr = parse_expression_no_errors(src);
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Bool(false))));
    }

    #[test]
    fn parses_integer_literal() {
        let src = "42";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_negative_integer_literal() {
        let src = "-42";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(negative);
    }

    #[test]
    fn parses_parenthesized_expression() {
        let src = "(42)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Parenthesized(expr) = expr.kind else {
            panic!("Expected parenthesized expression");
        };
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 42_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_unit() {
        let src = "()";
        let expr = parse_expression_no_errors(src);
        assert!(matches!(expr.kind, ExpressionKind::Literal(Literal::Unit)));
    }

    #[test]
    fn parses_str() {
        let src = "\"hello\"";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Str(string)) = expr.kind else {
            panic!("Expected string literal");
        };
        assert_eq!(string, "hello");
    }

    #[test]
    fn parses_raw_str() {
        let src = "r#\"hello\"#";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::RawStr(string, n)) = expr.kind else {
            panic!("Expected raw string literal");
        };
        assert_eq!(string, "hello");
        assert_eq!(n, 1);
    }

    #[test]
    fn parses_fmt_str() {
        let src = "f\"hello\"";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::FmtStr(fragments, length)) = expr.kind else {
            panic!("Expected format string literal");
        };
        assert_eq!(fragments[0].to_string(), "hello");
        assert_eq!(length, 5);
    }

    #[test]
    fn parses_tuple_expression() {
        let src = "(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Tuple(mut exprs) = expr.kind else {
            panic!("Expected tuple expression");
        };
        assert_eq!(exprs.len(), 2);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 2_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_block_expression_with_a_single_expression() {
        let src = "{ 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Block(mut block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 1);

        let statement = block.statements.remove(0);
        let StatementKind::Expression(expr) = statement.kind else {
            panic!("Expected expression statement");
        };

        let ExpressionKind::Literal(Literal::Integer(field, negative)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(field, 1_u128.into());
        assert!(!negative);
    }

    #[test]
    fn parses_block_expression_with_multiple_statements() {
        let src = "
        {
            let x = 1;
            let y = 2;
            3
        }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 3);
        assert_eq!(block.statements[0].kind.to_string(), "let x = 1");
        assert_eq!(block.statements[1].kind.to_string(), "let y = 2");
        assert_eq!(block.statements[2].kind.to_string(), "3");
    }

    #[test]
    fn parses_block_expression_adds_semicolons() {
        let src = "
        {
            1
            2
            3
        }";
        let mut parser = Parser::for_str(src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.span.end() as usize, src.len());
        assert_eq!(parser.errors.len(), 2);
        assert!(matches!(
            parser.errors[0].reason(),
            Some(ParserErrorReason::MissingSeparatingSemi)
        ));
        assert!(matches!(
            parser.errors[1].reason(),
            Some(ParserErrorReason::MissingSeparatingSemi)
        ));
        let ExpressionKind::Block(block) = expr.kind else {
            panic!("Expected block expression");
        };
        assert_eq!(block.statements.len(), 3);
    }

    #[test]
    fn parses_unsafe_expression() {
        let src = "
        /// Safety: test
        unsafe { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Unsafe(block, _) = expr.kind else {
            panic!("Expected unsafe expression");
        };
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn parses_unsafe_expression_with_doc_comment() {
        let src = "
        /// Safety: test
        unsafe { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Unsafe(block, _) = expr.kind else {
            panic!("Expected unsafe expression");
        };
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn parses_unclosed_parentheses() {
        let src = "
        (
        ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected an expression but found end of input");
    }

    #[test]
    fn parses_missing_comma_in_tuple() {
        let src = "
        (1 2)
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let reason = get_single_error_reason(&parser.errors, span);
        let ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items } = reason else {
            panic!("Expected a different error");
        };
        assert_eq!(token, &Token::Comma);
        assert_eq!(*items, "expressions");
    }

    #[test]
    fn parses_empty_array_expression() {
        let src = "[]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert!(exprs.is_empty());
    }

    #[test]
    fn parses_array_expression_with_one_element() {
        let src = "[1]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].to_string(), "1");
    }

    #[test]
    fn parses_array_expression_with_two_elements() {
        let src = "[1, 3]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0].to_string(), "1");
        assert_eq!(exprs[1].to_string(), "3");
    }

    #[test]
    fn parses_array_expression_with_two_elements_missing_comma() {
        let src = "
        [1 3]
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.span.end() as usize, src.len());

        let reason = get_single_error_reason(&parser.errors, span);
        let ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items } = reason else {
            panic!("Expected a different error");
        };
        assert_eq!(token, &Token::Comma);
        assert_eq!(*items, "expressions");

        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(exprs.len(), 2);
        assert_eq!(exprs[0].to_string(), "1");
        assert_eq!(exprs[1].to_string(), "3");
    }

    #[test]
    fn parses_repeated_array_expression() {
        let src = "[1; 10]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Array(ArrayLiteral::Repeated {
            repeated_element,
            length,
        })) = expr.kind
        else {
            panic!("Expected array literal");
        };
        assert_eq!(repeated_element.to_string(), "1");
        assert_eq!(length.to_string(), "10");
    }

    #[test]
    fn parses_empty_slice_expression() {
        let src = "&[]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Slice(ArrayLiteral::Standard(exprs))) = expr.kind
        else {
            panic!("Expected slice literal");
        };
        assert!(exprs.is_empty());
    }

    #[test]
    fn parses_variable_ident() {
        let src = "foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_variable_path() {
        let src = "foo::bar";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo::bar");
    }

    #[test]
    fn parses_variable_path_with_turbofish() {
        let src = "foo::<9>";
        parse_expression_no_errors(src);
    }

    #[test]
    fn parses_mutable_ref() {
        let src = "&mut foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::MutableReference));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_minus() {
        let src = "-foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Minus));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_not() {
        let src = "!foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Not));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_dereference() {
        let src = "*foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Dereference { implicitly_added: false }));

        let ExpressionKind::Variable(path) = prefix.rhs.kind else {
            panic!("Expected variable");
        };
        assert_eq!(path.to_string(), "foo");
    }

    #[test]
    fn parses_quote() {
        let src = "quote { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Quote(tokens) = expr.kind else {
            panic!("Expected quote expression");
        };
        assert_eq!(tokens.0.len(), 1);
    }

    #[test]
    fn parses_call() {
        let src = "foo(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo");
        assert_eq!(call.arguments.len(), 2);
        assert!(!call.is_macro_call);
    }

    #[test]
    fn parses_call_missing_comma() {
        let src = "
        foo(1 2)
              ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.span.end() as usize, src.len());
        let reason = get_single_error_reason(&parser.errors, span);
        let ParserErrorReason::ExpectedTokenSeparatingTwoItems { token, items } = reason else {
            panic!("Expected a different error");
        };
        assert_eq!(token, &Token::Comma);
        assert_eq!(*items, "arguments");

        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo");
        assert_eq!(call.arguments.len(), 2);
        assert!(!call.is_macro_call);
    }

    #[test]
    fn parses_call_with_wrong_expression() {
        let src = "foo(]) ";
        let mut parser = Parser::for_str(src);
        parser.parse_expression_or_error();
        assert!(!parser.errors.is_empty());
    }

    #[test]
    fn parses_call_with_turbofish() {
        let src = "foo::<T>(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo::<T>");
        assert_eq!(call.arguments.len(), 2);
        assert!(!call.is_macro_call);
    }

    #[test]
    fn parses_macro_call() {
        let src = "foo!(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Call(call) = expr.kind else {
            panic!("Expected call expression");
        };
        assert_eq!(call.func.to_string(), "foo");
        assert_eq!(call.arguments.len(), 2);
        assert!(call.is_macro_call);
    }

    #[test]
    fn parses_member_access() {
        let src = "foo.bar";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::MemberAccess(member_access) = expr.kind else {
            panic!("Expected member access expression");
        };
        assert_eq!(member_access.lhs.to_string(), "foo");
        assert_eq!(member_access.rhs.to_string(), "bar");
    }

    #[test]
    fn parses_method_call() {
        let src = "foo.bar(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(!method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert!(method_call.generics.is_none());
    }

    #[test]
    fn parses_method_call_with_turbofish() {
        let src = "foo.bar::<T, U>(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(!method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert_eq!(method_call.generics.unwrap().len(), 2);
    }

    #[test]
    fn parses_method_macro_call() {
        let src = "foo.bar!(1, 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::MethodCall(method_call) = expr.kind else {
            panic!("Expected method call expression");
        };
        assert_eq!(method_call.object.to_string(), "foo");
        assert_eq!(method_call.method_name.to_string(), "bar");
        assert!(method_call.is_macro_call);
        assert_eq!(method_call.arguments.len(), 2);
        assert!(method_call.generics.is_none());
    }

    #[test]
    fn parses_empty_constructor() {
        let src = "Foo {}";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Constructor(constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert!(constructor.fields.is_empty());
    }

    #[test]
    fn parses_constructor_with_fields() {
        let src = "Foo { x: 1, y, z: 2 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Constructor(mut constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert_eq!(constructor.fields.len(), 3);

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "x");
        assert_eq!(expr.to_string(), "1");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "y");
        assert_eq!(expr.to_string(), "y");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "z");
        assert_eq!(expr.to_string(), "2");
    }

    #[test]
    fn parses_constructor_with_fields_recovers_if_assign_instead_of_colon() {
        let src = "
        Foo { x = 1, y }
                ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let expr = parser.parse_expression_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a ':' but found '='");

        let ExpressionKind::Constructor(mut constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert_eq!(constructor.fields.len(), 2);

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "x");
        assert_eq!(expr.to_string(), "1");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "y");
        assert_eq!(expr.to_string(), "y");
    }

    #[test]
    fn parses_constructor_recovers_if_double_colon_instead_of_colon() {
        let src = "
        Foo { x: 1, y:: z }
                     ^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        let expr = parser.parse_expression_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a ':' but found '::'");

        let ExpressionKind::Constructor(mut constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert_eq!(constructor.fields.len(), 2);

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "x");
        assert_eq!(expr.to_string(), "1");

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "y");
        assert_eq!(expr.to_string(), "z");
    }

    #[test]
    fn parses_parses_if_true() {
        let src = "if true { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        let ExpressionKind::Block(block_expr) = if_expr.consequence.kind else {
            panic!("Expected block");
        };
        assert_eq!(block_expr.statements.len(), 1);
        assert_eq!(block_expr.statements[0].kind.to_string(), "1");
        assert!(if_expr.alternative.is_none());
    }

    #[test]
    fn parses_parses_if_var() {
        let src = "if foo { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "foo");
    }

    #[test]
    fn parses_parses_if_else() {
        let src = "if true { 1 } else { 2 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        assert!(if_expr.alternative.is_some());
    }

    #[test]
    fn parses_parses_if_else_if() {
        let src = "if true { 1 } else if false { 2 } else { 3 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::If(if_expr) = expr.kind else {
            panic!("Expected if");
        };
        assert_eq!(if_expr.condition.to_string(), "true");
        let ExpressionKind::If(..) = if_expr.alternative.unwrap().kind else {
            panic!("Expected if");
        };
    }

    #[test]
    fn parses_cast() {
        let src = "1 as u8";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Cast(cast_expr) = expr.kind else {
            panic!("Expected cast");
        };
        assert_eq!(cast_expr.lhs.to_string(), "1");
        assert_eq!(cast_expr.r#type.to_string(), "u8");
    }

    #[test]
    fn parses_cast_missing_type() {
        let src = "
        1 as
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str(&src);
        parser.parse_expression();
        let error = get_single_error(&parser.errors, span);
        assert_eq!(error.to_string(), "Expected a type but found end of input");
    }

    #[test]
    fn parses_index() {
        let src = "1[2]";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Index(index_expr) = expr.kind else {
            panic!("Expected index");
        };
        assert_eq!(index_expr.collection.to_string(), "1");
        assert_eq!(index_expr.index.to_string(), "2");
    }

    #[test]
    fn parses_operators() {
        for operator in BinaryOpKind::iter() {
            let src = format!("1 {operator} 2");
            let mut parser = Parser::for_str(&src);
            let expr = parser.parse_expression_or_error();
            assert_eq!(expr.span.end() as usize, src.len());
            assert!(parser.errors.is_empty(), "Expected no errors for {operator}");
            let ExpressionKind::Infix(infix_expr) = expr.kind else {
                panic!("Expected infix for {operator}");
            };
            assert_eq!(infix_expr.lhs.to_string(), "1");
            assert_eq!(infix_expr.operator.contents, operator);
            assert_eq!(infix_expr.rhs.to_string(), "2");
        }
    }

    #[test]
    fn parses_operator_precedence() {
        // This test produces a gigantic expression with lots of infix expressions without parentheses.
        // We parse it, then we transform that to a string. Because `InfixExpression::to_string()` adds parentheses
        // around it, we can check the operator precedence is correct by checking where parentheses were placed.
        let multiply_or_divide_or_modulo = "1 * 2 / 3 % 4";
        let expected_multiply_or_divide_or_modulo = "(((1 * 2) / 3) % 4)";

        let add_or_subtract = format!("{multiply_or_divide_or_modulo} + {multiply_or_divide_or_modulo} - {multiply_or_divide_or_modulo}");
        let expected_add_or_subtract = format!("(({expected_multiply_or_divide_or_modulo} + {expected_multiply_or_divide_or_modulo}) - {expected_multiply_or_divide_or_modulo})");

        let shift = format!("{add_or_subtract} << {add_or_subtract} >> {add_or_subtract}");
        let expected_shift = format!("(({expected_add_or_subtract} << {expected_add_or_subtract}) >> {expected_add_or_subtract})");

        let less_or_greater = format!("{shift} < {shift} > {shift} <= {shift} >= {shift}");
        let expected_less_or_greater = format!("(((({expected_shift} < {expected_shift}) > {expected_shift}) <= {expected_shift}) >= {expected_shift})");

        let xor = format!("{less_or_greater} ^ {less_or_greater}");
        let expected_xor = format!("({expected_less_or_greater} ^ {expected_less_or_greater})");

        let and = format!("{xor} & {xor}");
        let expected_and = format!("({expected_xor} & {expected_xor})");

        let or = format!("{and} | {and}");
        let expected_or = format!("({expected_and} | {expected_and})");

        let equal_or_not_equal = format!("{or} == {or} != {or}");
        let expected_equal_or_not_equal =
            format!("(({expected_or} == {expected_or}) != {expected_or})");

        let src = &equal_or_not_equal;
        let expected_src = expected_equal_or_not_equal;

        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Infix(infix_expr) = expr.kind else {
            panic!("Expected infix");
        };
        assert_eq!(infix_expr.to_string(), expected_src);
    }

    #[test]
    fn parses_empty_lambda() {
        let src = "|| 1";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Lambda(lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert!(lambda.parameters.is_empty());
        assert_eq!(lambda.body.to_string(), "1");
        assert!(matches!(lambda.return_type.typ, UnresolvedTypeData::Unspecified));
    }

    #[test]
    fn parses_lambda_with_arguments() {
        let src = "|x, y: Field| 1";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Lambda(mut lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert_eq!(lambda.parameters.len(), 2);

        let (pattern, typ) = lambda.parameters.remove(0);
        assert_eq!(pattern.to_string(), "x");
        assert!(matches!(typ.typ, UnresolvedTypeData::Unspecified));

        let (pattern, typ) = lambda.parameters.remove(0);
        assert_eq!(pattern.to_string(), "y");
        assert!(matches!(typ.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_lambda_with_return_type() {
        let src = "|| -> Field 1";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Lambda(lambda) = expr.kind else {
            panic!("Expected lambda");
        };
        assert!(lambda.parameters.is_empty());
        assert_eq!(lambda.body.to_string(), "1");
        assert!(matches!(lambda.return_type.typ, UnresolvedTypeData::FieldElement));
    }

    #[test]
    fn parses_as_trait_path() {
        let src = "<Field as foo::Bar>::baz";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::AsTraitPath(as_trait_path) = expr.kind else {
            panic!("Expected as_trait_path")
        };
        assert_eq!(as_trait_path.typ.typ.to_string(), "Field");
        assert_eq!(as_trait_path.trait_path.to_string(), "foo::Bar");
        assert!(as_trait_path.trait_generics.is_empty());
        assert_eq!(as_trait_path.impl_item.to_string(), "baz");
    }

    #[test]
    fn parses_comptime_expression() {
        let src = "comptime { 1 }";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Comptime(block, _) = expr.kind else {
            panic!("Expected comptime block");
        };
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn parses_type_path() {
        let src = "Field::foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::TypePath(type_path) = expr.kind else {
            panic!("Expected type_path");
        };
        assert_eq!(type_path.typ.to_string(), "Field");
        assert_eq!(type_path.item.to_string(), "foo");
        assert!(type_path.turbofish.is_none());
    }

    #[test]
    fn parses_type_path_with_generics() {
        let src = "Field::foo::<T>";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::TypePath(type_path) = expr.kind else {
            panic!("Expected type_path");
        };
        assert_eq!(type_path.typ.to_string(), "Field");
        assert_eq!(type_path.item.to_string(), "foo");
        assert!(type_path.turbofish.is_some());
    }

    #[test]
    fn parses_unquote_var() {
        let src = "$foo::bar";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Unquote(expr) = expr.kind else {
            panic!("Expected unquote");
        };
        let ExpressionKind::Variable(path) = expr.kind else {
            panic!("Expected unquote");
        };
        assert_eq!(path.to_string(), "foo::bar");
    }

    #[test]
    fn parses_unquote_expr() {
        let src = "$(1 + 2)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Unquote(expr) = expr.kind else {
            panic!("Expected unquote");
        };
        assert_eq!(expr.kind.to_string(), "((1 + 2))");
    }
}
