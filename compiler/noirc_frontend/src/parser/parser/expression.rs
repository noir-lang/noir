use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{
        ArrayLiteral, BlockExpression, CallExpression, CastExpression, ConstrainExpression,
        ConstrainKind, ConstructorExpression, Expression, ExpressionKind, Ident, IfExpression,
        IndexExpression, Literal, MatchExpression, MemberAccessExpression, MethodCallExpression,
        Statement, TypePath, UnaryOp, UnresolvedType, UnresolvedTypeData, UnsafeExpression,
    },
    parser::{ParserErrorReason, labels::ParsingRuleLabel, parser::parse_many::separated_by_comma},
    token::{Keyword, Token, TokenKind},
};

use super::{
    MAX_PARSER_RECURSION_DEPTH, Parser,
    parse_many::{
        separated_by_comma_until_right_brace, separated_by_comma_until_right_paren,
        without_separator,
    },
};

/// When parsing an array literal we might bump into `[expr; length]::ident()`,
/// where the user expected to call a method on an array type.
/// That actually needs to be written as `<[expr; length]>::ident()`, so
/// in that case we'll produce an error and return `ArrayLiteralOrError::Error`.
enum ArrayLiteralOrError {
    ArrayLiteral(ArrayLiteral),
    Error,
}

impl Parser<'_> {
    #[inline(always)]
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

    #[inline(always)]
    pub(crate) fn parse_expression_or_error_impl(
        &mut self,
        allow_constructors: bool,
    ) -> Expression {
        if let Some(expr) = self.parse_expression_impl(allow_constructors) {
            expr
        } else {
            self.push_expected_expression();
            Expression {
                kind: ExpressionKind::Error,
                location: self.location_at_previous_token_end(),
            }
        }
    }

    fn parse_expression_impl(&mut self, allow_constructors: bool) -> Option<Expression> {
        // Check recursion depth to prevent stack overflow
        if self.recursion_depth >= MAX_PARSER_RECURSION_DEPTH {
            self.push_error(
                ParserErrorReason::MaximumRecursionDepthExceeded,
                self.current_token_location,
            );
            // Skip to a recovery point to avoid cascading errors
            self.skip_to_recovery_point();
            // Set flag to suppress cascading errors during stack unwinding
            self.recovering_from_depth_overflow = true;
            return None;
        }

        self.recursion_depth += 1;
        let result = self.parse_equal_or_not_equal(allow_constructors);
        self.recursion_depth -= 1;

        // Clear recovery flag when we've fully unwound (back at top level)
        if self.recursion_depth == 0 {
            self.recovering_from_depth_overflow = false;
        }

        result
    }

    /// Term
    ///    = UnaryExpression
    ///    | CastExpression
    pub(super) fn parse_term(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_location = self.current_token_location;

        let mut term = self.parse_unary(allow_constructors)?;
        let mut parsed;

        loop {
            (term, parsed) = self.parse_cast(term, start_location);
            if !parsed {
                break;
            }
        }

        Some(term)
    }

    /// UnaryExpression
    ///    = UnaryOp* Atom
    fn parse_unary(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_location = self.current_token_location;

        if let Some(operator) = self.parse_unary_op() {
            let Some(rhs) = self.parse_unary(allow_constructors) else {
                self.expected_label(ParsingRuleLabel::Expression);
                return None;
            };
            let kind = ExpressionKind::prefix(operator, rhs);
            let location = self.location_since(start_location);
            return Some(Expression { kind, location });
        }

        self.parse_atom(allow_constructors)
    }

    /// UnaryOp = '&' 'mut' | '-' | '!' | '*'
    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        if self.at(Token::Ampersand) {
            let mut mutable = false;
            if self.next_is(Token::Keyword(Keyword::Mut)) {
                mutable = true;
                self.bump();
            }
            self.bump();
            Some(UnaryOp::Reference { mutable })
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

    /// Atom
    ///     = Quark AtomRhs*
    fn parse_atom(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_location = self.current_token_location;
        let mut atom = self.parse_quark(allow_constructors)?;
        let mut parsed;

        loop {
            (atom, parsed) = self.parse_atom_rhs(atom, start_location);
            if parsed {
                continue;
            } else {
                break;
            }
        }

        Some(atom)
    }

    /// AtomRhs
    ///     = CallExpression
    ///     | MemberAccessOrMethodCallExpression
    ///     | IndexExpression
    fn parse_atom_rhs(
        &mut self,
        mut atom: Expression,
        start_location: Location,
    ) -> (Expression, bool) {
        let mut parsed;

        (atom, parsed) = self.parse_call(atom, start_location);
        if parsed {
            return (atom, parsed);
        }

        (atom, parsed) = self.parse_member_access_or_method_call(atom, start_location);
        if parsed {
            return (atom, parsed);
        }

        self.parse_index(atom, start_location)
    }

    pub(super) fn parse_member_accesses_or_method_calls_after_expression(
        &mut self,
        mut atom: Expression,
        start_location: Location,
    ) -> Expression {
        let mut parsed;

        loop {
            (atom, parsed) = self.parse_member_access_or_method_call(atom, start_location);
            if parsed {
                continue;
            } else {
                break;
            }
        }

        atom
    }

    /// CallExpression = Quark CallArguments
    fn parse_call(&mut self, atom: Expression, start_location: Location) -> (Expression, bool) {
        if let Some(call_arguments) = self.parse_call_arguments() {
            let kind = ExpressionKind::Call(Box::new(CallExpression {
                func: Box::new(atom),
                arguments: call_arguments.arguments,
                is_macro_call: call_arguments.is_macro_call,
            }));
            let location = self.location_since(start_location);
            let atom = Expression { kind, location };
            (atom, true)
        } else {
            (atom, false)
        }
    }

    /// MemberAccessOrMethodCallExpression
    ///     = MemberAccessExpression
    ///     | MethodCallExpression
    ///
    /// MemberAccessExpression = Quark '.' identifier
    ///
    /// MethodCallExpression = Quark '.' identifier CallArguments
    fn parse_member_access_or_method_call(
        &mut self,
        atom: Expression,
        start_location: Location,
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

        let location = self.location_since(start_location);
        let atom = Expression { kind, location };
        (atom, true)
    }

    fn parse_member_access_field_name(&mut self) -> Option<Ident> {
        if let Some(ident) = self.eat_ident() {
            Some(ident)
        // We allow integer type suffixes on tuple field names since this lets
        // users unquote typed integers in macros to use as a tuple access expression.
        // See https://github.com/noir-lang/noir/pull/10330#issuecomment-3499399843
        } else if let Some((int, _)) = self.eat_int() {
            Some(Ident::new(int.to_string(), self.previous_token_location))
        } else {
            self.push_error(
                ParserErrorReason::ExpectedFieldName(self.token.token().clone()),
                self.current_token_location,
            );
            None
        }
    }

    /// CastExpression = UnaryExpression 'as' Type
    fn parse_cast(&mut self, atom: Expression, start_location: Location) -> (Expression, bool) {
        if !self.eat_keyword(Keyword::As) {
            return (atom, false);
        }

        // Here we don't allow generics on a type so that `x as u8 < 3` parses without error.
        // In Rust the above is a syntax error as `u8<` would denote a generic type.
        // In Noir it's unlikely we'd want generic types in casts and, to avoid a breaking change,
        // we disallow generics in that position.
        let typ = self.parse_type_or_error_without_generics();
        let kind = ExpressionKind::Cast(Box::new(CastExpression { lhs: atom, r#type: typ }));
        let location = self.location_since(start_location);
        let atom = Expression { kind, location };
        (atom, true)
    }

    /// IndexExpression = Quark '[' Expression ']'
    fn parse_index(&mut self, atom: Expression, start_location: Location) -> (Expression, bool) {
        if !self.eat_left_bracket() {
            return (atom, false);
        }

        let index = self.parse_expression_or_error();
        self.eat_or_error(Token::RightBracket);
        let kind = ExpressionKind::Index(Box::new(IndexExpression { collection: atom, index }));
        let location = self.location_since(start_location);
        let atom = Expression { kind, location };
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

    /// Quark
    ///     = Literal
    ///     | ParenthesesExpression
    ///     | UnsafeExpression
    ///     | PathExpression
    ///     | IfExpression
    ///     | Lambda
    ///     | ComptimeExpression
    ///     | UnquoteExpression
    ///     | TypePathExpression
    ///     | NamelessTypePathExpression
    ///     | AsTraitPath
    ///     | ResolvedExpression
    ///     | InternedExpression
    ///     | InternedStatementExpression
    fn parse_quark(&mut self, allow_constructors: bool) -> Option<Expression> {
        let start_location = self.current_token_location;
        let kind = self.parse_quark_kind(allow_constructors)?;
        Some(Expression { kind, location: self.location_since(start_location) })
    }

    fn parse_quark_kind(&mut self, allow_constructors: bool) -> Option<ExpressionKind> {
        // Like in Rust, we allow parsing doc comments on top of an expression but they always produce a warning.
        self.warn_on_outer_doc_comments();

        if let Some(kind) = self.parse_unsafe_expr() {
            return Some(kind);
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
        if matches!(
            self.token.token(),
            Token::InternedUnresolvedTypeData(..) | Token::QuotedType(..)
        ) && self.next_is(Token::LeftBrace)
        {
            let location = self.current_token_location;
            let typ = self.parse_interned_type().or_else(|| self.parse_resolved_type()).unwrap();
            self.eat_or_error(Token::LeftBrace);
            let typ = UnresolvedType { typ, location };
            return Some(self.parse_constructor(typ));
        }

        if let Some(kind) = self.parse_if_expr() {
            return Some(kind);
        }

        if let Some(kind) = self.parse_match_expr() {
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

        if let Some(kind) = self.parse_nameless_type_path_or_as_trait_path_type_expression() {
            return Some(kind);
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

    /// NamelessTypePathExpression = '<' Type '>' '::' identifier ( '::' GenericTypeArgs )?
    fn parse_nameless_type_path_or_as_trait_path_type_expression(
        &mut self,
    ) -> Option<ExpressionKind> {
        if !self.eat_less() {
            return None;
        }

        let typ = self.parse_type_or_error();
        if self.eat_keyword(Keyword::As) {
            let as_trait_path = self.parse_as_trait_path_for_type_after_as_keyword(typ);
            Some(ExpressionKind::AsTraitPath(Box::new(as_trait_path)))
        } else {
            self.eat_or_error(Token::Greater);
            let type_path = self.parse_type_path_expr_for_type(typ);
            Some(ExpressionKind::TypePath(Box::new(type_path)))
        }
    }

    /// ResolvedExpression = unquote_marker
    fn parse_resolved_expr(&mut self) -> Option<ExpressionKind> {
        Some(ExpressionKind::Resolved(self.eat_unquote_marker()?))
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
    fn parse_unsafe_expr(&mut self) -> Option<ExpressionKind> {
        let start_location = self.current_token_location;
        let comments_before_unsafe = self.current_token_comments.clone();

        if !self.eat_keyword(Keyword::Unsafe) {
            return None;
        }

        let comments: &str = if comments_before_unsafe.is_empty() {
            if let Some(statement_comments) = &self.statement_comments {
                statement_comments
            } else {
                ""
            }
        } else {
            &comments_before_unsafe
        };

        if !comments.lines().any(|line| line.trim().to_lowercase().starts_with("safety:")) {
            self.push_error(ParserErrorReason::MissingSafetyComment, start_location);
        }

        if let Some(block) = self.parse_block() {
            Some(ExpressionKind::Unsafe(UnsafeExpression {
                block,
                unsafe_keyword_location: start_location,
            }))
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

        ExpressionKind::Constructor(Box::new(ConstructorExpression { typ, fields }))
    }

    fn parse_constructor_field(&mut self) -> Option<(Ident, Expression)> {
        // Loop to do some error recovery
        loop {
            // Make sure not to loop forever
            if self.at_eof() || self.at(Token::RightBrace) {
                return None;
            }

            // If we can't find an identifier, error but continue looking for an identifier
            if !matches!(self.token.token(), Token::Ident(..)) {
                self.expected_identifier();
                self.bump();
                // Don't error again if a comma comes next
                if self.at(Token::Comma) {
                    self.bump();
                }
                continue;
            }

            let ident = self.eat_ident()?;

            return Some(if self.eat_colon() {
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
            });
        }
    }

    /// IfExpression = 'if' ExpressionExceptConstructor Block ( 'else' ( Block | IfExpression ) )?
    pub(super) fn parse_if_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::If) {
            return None;
        }

        let condition = self.parse_expression_except_constructor_or_error();

        let start_location = self.current_token_location;
        let Some(consequence) = self.parse_block() else {
            self.expected_token(Token::LeftBrace);
            let location = self.location_at_previous_token_end();
            return Some(ExpressionKind::If(Box::new(IfExpression {
                condition,
                consequence: Expression { kind: ExpressionKind::Error, location },
                alternative: None,
            })));
        };
        let location = self.location_since(start_location);
        let consequence = Expression { kind: ExpressionKind::Block(consequence), location };

        let alternative = if self.eat_keyword(Keyword::Else) {
            let start_location = self.current_token_location;
            if let Some(block) = self.parse_block() {
                let location = self.location_since(start_location);
                Some(Expression { kind: ExpressionKind::Block(block), location })
            } else if let Some(if_expr) = self.parse_if_expr() {
                Some(Expression { kind: if_expr, location: self.location_since(start_location) })
            } else {
                self.expected_token(Token::LeftBrace);
                None
            }
        } else {
            None
        };

        Some(ExpressionKind::If(Box::new(IfExpression { condition, consequence, alternative })))
    }

    /// MatchExpression = 'match' ExpressionExceptConstructor '{' MatchRule* '}'
    pub(super) fn parse_match_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::Match) {
            return None;
        }

        let expression = self.parse_expression_except_constructor_or_error();

        if !self.eat_left_brace() {
            self.expected_token(Token::LeftBrace);
            return Some(ExpressionKind::Error);
        }

        let rules = self.parse_many(
            "match cases",
            without_separator().until(Token::RightBrace),
            Self::parse_match_rule,
        );

        Some(ExpressionKind::Match(Box::new(MatchExpression { expression, rules })))
    }

    /// MatchRule = Expression '=>' (Block ','?) | (Expression ',')
    fn parse_match_rule(&mut self) -> Option<(Expression, Expression)> {
        let pattern = self.parse_expression()?;
        self.eat_or_error(Token::FatArrow);

        let start_location = self.current_token_location;
        let branch = match self.parse_block() {
            Some(block) => {
                let location = self.location_since(start_location);
                let block = Expression::new(ExpressionKind::Block(block), location);
                self.eat_comma(); // comma is optional if we have a block
                block
            }
            None => {
                let branch = self.parse_expression_or_error();
                self.eat_or_error(Token::Comma);
                branch
            }
        };
        Some((pattern, branch))
    }

    /// ComptimeExpression = 'comptime' Block
    fn parse_comptime_expr(&mut self) -> Option<ExpressionKind> {
        if !self.eat_keyword(Keyword::Comptime) {
            return None;
        }

        let start_location = self.current_token_location;

        let Some(block) = self.parse_block() else {
            self.expected_token(Token::LeftBrace);
            return None;
        };

        Some(ExpressionKind::Comptime(block, self.location_since(start_location)))
    }

    /// UnquoteExpression
    ///     = '$' identifier
    ///     | '$' '(' Expression ')'
    fn parse_unquote_expr(&mut self) -> Option<ExpressionKind> {
        let start_location = self.current_token_location;

        if !self.eat(Token::DollarSign) {
            return None;
        }

        if let Some(path) = self.parse_path() {
            let expr = Expression {
                kind: ExpressionKind::Variable(path),
                location: self.location_since(start_location),
            };
            return Some(ExpressionKind::Unquote(Box::new(expr)));
        }

        let location_at_left_paren = self.current_token_location;
        if self.eat_left_paren() {
            let expr = self.parse_expression_or_error();
            self.eat_or_error(Token::RightParen);
            let expr = Expression {
                kind: ExpressionKind::Parenthesized(Box::new(expr)),
                location: self.location_since(location_at_left_paren),
            };
            return Some(ExpressionKind::Unquote(Box::new(expr)));
        }

        self.push_error(
            ParserErrorReason::ExpectedIdentifierOrLeftParenAfterDollar,
            self.current_token_location,
        );

        None
    }

    /// TypePathExpression = PrimitiveType '::' identifier ( '::' GenericTypeArgs )?
    fn parse_type_path_expr(&mut self) -> Option<ExpressionKind> {
        let start_location = self.current_token_location;
        let typ = self.parse_primitive_type()?;
        let location = self.location_since(start_location);
        let typ = UnresolvedType { typ, location };

        if self.at(Token::DoubleColon) {
            Some(ExpressionKind::TypePath(Box::new(self.parse_type_path_expr_for_type(typ))))
        } else {
            // This is the case when we find `Field` or `i32` but `::` doesn't follow it.
            self.push_error(ParserErrorReason::ExpectedValueFoundBuiltInType { typ }, location);
            Some(ExpressionKind::Error)
        }
    }

    fn parse_type_path_expr_for_type(&mut self, typ: UnresolvedType) -> TypePath {
        self.eat_or_error(Token::DoubleColon);

        let item = if let Some(ident) = self.eat_ident() {
            ident
        } else {
            self.expected_identifier();
            Ident::new(String::new(), self.location_at_previous_token_end())
        };

        let turbofish = self.eat_double_colon().then(|| {
            let generics = self.parse_generic_type_args();
            if generics.is_empty() {
                self.expected_token(Token::Less);
            }
            generics
        });

        TypePath { typ, item, turbofish }
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
    ///     | ConstrainExpression
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

        if let Some((int, suffix)) = self.eat_int() {
            return Some(ExpressionKind::integer(int, suffix));
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

        if let Some(literal_or_error) = self.parse_array_literal() {
            return match literal_or_error {
                ArrayLiteralOrError::ArrayLiteral(literal) => {
                    Some(ExpressionKind::Literal(Literal::Array(literal)))
                }
                ArrayLiteralOrError::Error => Some(ExpressionKind::Error),
            };
        }

        if let Some(literal_or_error) = self.parse_slice_literal() {
            return match literal_or_error {
                ArrayLiteralOrError::ArrayLiteral(literal) => {
                    Some(ExpressionKind::Literal(Literal::Slice(literal)))
                }
                ArrayLiteralOrError::Error => Some(ExpressionKind::Error),
            };
        }

        if let Some(kind) = self.parse_block() {
            return Some(ExpressionKind::Block(kind));
        }

        if let Some(constrain) = self.parse_constrain_expression() {
            return Some(ExpressionKind::Constrain(constrain));
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
    fn parse_array_literal(&mut self) -> Option<ArrayLiteralOrError> {
        let start_location = self.current_token_location;
        let errors_before_array = self.errors.len();

        if !self.eat_left_bracket() {
            return None;
        }

        if self.eat_right_bracket() {
            return Some(ArrayLiteralOrError::ArrayLiteral(ArrayLiteral::Standard(Vec::new())));
        }

        let first_expr = self.parse_expression_or_error();

        if self.eat_semicolon() {
            let length = self.parse_expression_or_error();
            self.eat_or_error(Token::RightBracket);

            // If it's `[expr; length]::ident`, give an error that it's missing `<...>`
            if self.at(Token::DoubleColon) && matches!(self.next_token.token(), Token::Ident(..)) {
                // Remove any errors that happened during `[...]` as it's likely they happened
                // because of the missing angle brackets.
                self.errors.truncate(errors_before_array);

                let location = self.location_since(start_location);
                self.push_error(ParserErrorReason::MissingAngleBrackets, location);

                // Skip `::` and the identifier
                self.bump();
                self.bump();

                return Some(ArrayLiteralOrError::Error);
            }

            return Some(ArrayLiteralOrError::ArrayLiteral(ArrayLiteral::Repeated {
                repeated_element: Box::new(first_expr),
                length: Box::new(length),
            }));
        }

        let comma_after_first_expr = self.eat_comma();
        let second_expr_location = self.current_token_location;

        let mut exprs = self.parse_many(
            "expressions",
            separated_by_comma().until(Token::RightBracket),
            Self::parse_expression_in_list,
        );

        if !exprs.is_empty() && !comma_after_first_expr {
            self.expected_token_separating_items(Token::Comma, "expressions", second_expr_location);
        }

        exprs.insert(0, first_expr);

        Some(ArrayLiteralOrError::ArrayLiteral(ArrayLiteral::Standard(exprs)))
    }

    /// SliceExpression = '&' ArrayLiteral
    fn parse_slice_literal(&mut self) -> Option<ArrayLiteralOrError> {
        if !(self.at(Token::SliceStart) && self.next_is(Token::LeftBracket)) {
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
        let start_location = self.current_token_location;
        let errors_before_parentheses = self.errors.len();

        if !self.eat_left_paren() {
            return None;
        }

        if self.eat_right_paren() {
            // If it's `()::ident`, parse it as a type path but produce an error saying it should be `<()>::ident`.
            if self.at(Token::DoubleColon) && matches!(self.next_token.token(), Token::Ident(..)) {
                let location = self.location_since(start_location);
                let typ = UnresolvedTypeData::Unit;
                let typ = UnresolvedType { typ, location };
                let type_path = self.parse_type_path_expr_for_type(typ);

                self.push_error(ParserErrorReason::MissingAngleBrackets, location);

                return Some(ExpressionKind::TypePath(Box::new(type_path)));
            }

            return Some(ExpressionKind::Literal(Literal::Unit));
        }

        let (mut exprs, trailing_comma) = self.parse_many_return_trailing_separator_if_any(
            "expressions",
            separated_by_comma_until_right_paren(),
            Self::parse_expression_in_list,
        );

        // If it's `(..)::ident`, give an error that it's missing `<...>`
        if self.at(Token::DoubleColon) && matches!(self.next_token.token(), Token::Ident(..)) {
            // Remove any errors that happened during `(...)` as it's likely they happened
            // because of the missing angle brackets.
            self.errors.truncate(errors_before_parentheses);

            let location = self.location_since(start_location);
            self.push_error(ParserErrorReason::MissingAngleBrackets, location);

            // Skip `::` and the identifier
            self.bump();
            self.bump();

            return Some(ExpressionKind::Error);
        }

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
            // Don't generate cascading errors when recovering from depth overflow
            if !self.recovering_from_depth_overflow {
                self.expected_label(ParsingRuleLabel::Expression);
            }
            None
        }
    }

    /// ConstrainExpression
    ///     = 'constrain' Expression
    ///     | 'assert' Arguments
    ///     | 'assert_eq' Arguments
    pub(super) fn parse_constrain_expression(&mut self) -> Option<ConstrainExpression> {
        let start_location = self.current_token_location;
        let kind = self.parse_constrain_kind()?;

        Some(match kind {
            ConstrainKind::Assert | ConstrainKind::AssertEq => {
                let arguments = self.parse_arguments();
                if arguments.is_none() {
                    self.expected_token(Token::LeftParen);
                }
                let arguments = arguments.unwrap_or_default();

                ConstrainExpression {
                    kind,
                    arguments,
                    location: self.location_since(start_location),
                }
            }
            ConstrainKind::Constrain => {
                self.push_error(
                    ParserErrorReason::ConstrainDeprecated,
                    self.previous_token_location,
                );

                let expression = self.parse_expression_or_error();
                ConstrainExpression {
                    kind,
                    arguments: vec![expression],
                    location: self.location_since(start_location),
                }
            }
        })
    }

    fn parse_constrain_kind(&mut self) -> Option<ConstrainKind> {
        if self.eat_keyword(Keyword::Assert) {
            Some(ConstrainKind::Assert)
        } else if self.eat_keyword(Keyword::AssertEq) {
            Some(ConstrainKind::AssertEq)
        } else if self.eat_keyword(Keyword::Constrain) {
            Some(ConstrainKind::Constrain)
        } else {
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

    fn parse_statement_in_block(&mut self) -> Option<(Statement, (Option<Token>, Location))> {
        if let Some(statement) = self.parse_statement() {
            Some(statement)
        } else {
            self.expected_label(ParsingRuleLabel::Statement);
            None
        }
    }

    fn check_statements_require_semicolon(
        &mut self,
        statements: Vec<(Statement, (Option<Token>, Location))>,
    ) -> Vec<Statement> {
        let last = statements.len().saturating_sub(1);
        let iter = statements.into_iter().enumerate();
        vecmap(iter, |(i, (statement, (semicolon, location)))| {
            statement
                .add_semicolon(semicolon, location, i == last, &mut |error| self.errors.push(error))
        })
    }

    pub(super) fn push_expected_expression(&mut self) {
        self.expected_label(ParsingRuleLabel::Expression);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use strum::IntoEnumIterator;

    use crate::{
        ast::{
            ArrayLiteral, BinaryOpKind, ConstrainKind, Expression, ExpressionKind, Literal,
            StatementKind, UnaryOp,
        },
        parse_program_with_dummy_file,
        parser::{
            Parser, ParserErrorReason,
            parser::tests::{
                expect_no_errors, get_single_error, get_single_error_reason,
                get_source_with_error_span,
            },
        },
        signed_field::SignedField,
        token::Token,
    };

    fn parse_expression_no_errors(src: &str) -> Expression {
        let mut parser = Parser::for_str_with_dummy_file(src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.location.span.end() as usize, src.len());
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
        let src = "42u32";
        let expr = parse_expression_no_errors(src);
        use crate::token::IntegerTypeSuffix::U32;
        let ExpressionKind::Literal(Literal::Integer(value, Some(U32))) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::positive(42_u128));
    }

    #[test]
    fn parses_negative_integer_literal() {
        let src = "-42";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Literal(Literal::Integer(value, None)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::negative(42_u128));
    }

    #[test]
    fn parses_double_negative_integer_literal() {
        let src = "--42";
        let expr = parse_expression_no_errors(src);
        // In the past we used to parse this as the literal 42 instead of a double negation
        assert!(matches!(expr.kind, ExpressionKind::Prefix(..)));
    }

    #[test]
    fn parses_parenthesized_expression() {
        let src = "(42)";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Parenthesized(expr) = expr.kind else {
            panic!("Expected parenthesized expression");
        };
        let ExpressionKind::Literal(Literal::Integer(value, None)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::positive(42_u128));
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
        let ExpressionKind::Literal(Literal::Integer(value, None)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::positive(1_u128));

        let expr = exprs.remove(0);
        let ExpressionKind::Literal(Literal::Integer(value, None)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::positive(2_u128));
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

        let ExpressionKind::Literal(Literal::Integer(value, None)) = expr.kind else {
            panic!("Expected integer literal");
        };
        assert_eq!(value, SignedField::positive(1_u128));
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
        let mut parser = Parser::for_str_with_dummy_file(src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.location.span.end() as usize, src.len());
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
    fn parses_block_expression_with_a_single_assignment() {
        let src = "{ x = 1 }";
        let _ = parse_expression_no_errors(src);
    }

    #[test]
    fn parses_block_expression_with_a_single_break() {
        let src = "{ break }";
        let _ = parse_expression_no_errors(src);
    }

    #[test]
    fn parses_block_expression_with_a_single_continue() {
        let src = "{ continue }";
        let _ = parse_expression_no_errors(src);
    }

    #[test]
    fn parses_block_expression_with_a_single_let() {
        let src = "
        { let x = 1 }
                  ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        parser.parse_expression();
        let reason = get_single_error_reason(&parser.errors, span);
        let ParserErrorReason::MissingSemicolonAfterLet = reason else {
            panic!("Expected a different error");
        };
    }

    #[test]
    fn parses_unsafe_expression() {
        let src = "
        // Safety: test
        unsafe { 1 }";
        let mut parser = Parser::for_str_with_dummy_file(src);
        let expr = parser.parse_expression_or_error();
        assert!(parser.errors.is_empty());
        let ExpressionKind::Unsafe(unsafe_expression) = expr.kind else {
            panic!("Expected unsafe expression");
        };
        assert_eq!(unsafe_expression.block.statements.len(), 1);
    }

    #[test]
    fn parses_unsafe_expression_with_doc_comment() {
        let src = "
        /// Safety: test
        unsafe { 1 }";

        let mut parser = Parser::for_str_with_dummy_file(src);
        let expr = parser.parse_expression().unwrap();
        let ExpressionKind::Unsafe(unsafe_expression) = expr.kind else {
            panic!("Expected unsafe expression");
        };
        assert_eq!(unsafe_expression.block.statements.len(), 1);
    }

    #[test]
    fn parses_unclosed_parentheses() {
        let src = "
        (
        ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        parser.parse_expression();
        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected an expression but found end of input");
    }

    #[test]
    fn parses_missing_comma_in_tuple() {
        let src = "
        (1 2)
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
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
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.location.span.end() as usize, src.len());

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
        assert!(matches!(prefix.operator, UnaryOp::Reference { mutable: true }));

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
    fn parses_not_call() {
        let src = "!foo()";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Prefix(prefix) = expr.kind else {
            panic!("Expected prefix expression");
        };
        assert!(matches!(prefix.operator, UnaryOp::Not));

        let ExpressionKind::Call(_) = prefix.rhs.kind else {
            panic!("Expected call");
        };
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
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();
        assert_eq!(expr.location.span.end() as usize, src.len());
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
        let mut parser = Parser::for_str_with_dummy_file(src);
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
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected a ':' but found '='");

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
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected a ':' but found '::'");

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
    fn parses_constructor_with_fields_recovers_if_not_an_identifier_1() {
        let src = "
        Foo { x: 1, 2 }
                    ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        let ExpressionKind::Constructor(mut constructor) = expr.kind else {
            panic!("Expected constructor");
        };
        assert_eq!(constructor.typ.to_string(), "Foo");
        assert_eq!(constructor.fields.len(), 1);

        let (name, expr) = constructor.fields.remove(0);
        assert_eq!(name.to_string(), "x");
        assert_eq!(expr.to_string(), "1");

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected an identifier but found '2'");
    }

    #[test]
    fn parses_constructor_with_fields_recovers_if_not_an_identifier_2() {
        let src = "
        Foo { x: 1, 2, y: 2 }
                    ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

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
        assert_eq!(expr.to_string(), "2");

        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected an identifier but found '2'");
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
    fn parses_cast_of_negated_literal_once() {
        let src = "-1 as u8";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Cast(cast_expr) = expr.kind else {
            panic!("Expected cast");
        };
        assert_eq!(cast_expr.lhs.to_string(), "-1");
        assert_eq!(cast_expr.r#type.to_string(), "u8");
    }

    #[test]
    fn parses_cast_of_negated_var_twice() {
        let src = "--x as u8";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Cast(cast_expr) = expr.kind else {
            panic!("Expected cast");
        };
        let ExpressionKind::Prefix(prefix1) = cast_expr.lhs.kind else {
            panic!("Expected prefix expression");
        };
        assert_eq!(prefix1.operator, UnaryOp::Minus);
        let ExpressionKind::Prefix(prefix2) = prefix1.rhs.kind else {
            panic!("Expected prefix expression");
        };
        assert_eq!(prefix2.operator, UnaryOp::Minus);
        assert_eq!(prefix2.rhs.to_string(), "x");
        assert_eq!(cast_expr.r#type.to_string(), "u8");
    }

    #[test]
    fn parses_sum_then_cast() {
        let src = "1 + 2 as u8";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Infix(_) = expr.kind else {
            panic!("Expected infix");
        };
    }

    #[test]
    fn parses_cast_missing_type() {
        let src = "
        1 as
           ^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        parser.parse_expression();
        let error = get_single_error(&parser.errors, span);
        assert_snapshot!(error.to_string(), @"Expected a type but found end of input");
    }

    #[test]
    fn parses_cast_comparison() {
        // Note: in Rust this is a syntax error because `u8 <` is parsed as a generic type reference.
        // In Noir we allow this syntax, for now, mainly to avoid a breaking change and because
        // it's unlikely we'd want generic types in this context.
        let src = "1 as u8 < 3";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::Infix(infix_expr) = expr.kind else {
            panic!("Expected infix");
        };
        let ExpressionKind::Cast(cast_expr) = infix_expr.lhs.kind else {
            panic!("Expected cast");
        };
        assert_eq!(cast_expr.lhs.to_string(), "1");
        assert_eq!(cast_expr.r#type.to_string(), "u8");

        assert_eq!(infix_expr.rhs.to_string(), "3");
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
            let mut parser = Parser::for_str_with_dummy_file(&src);
            let expr = parser.parse_expression_or_error();
            assert_eq!(expr.location.span.end() as usize, src.len());
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

        let add_or_subtract = format!(
            "{multiply_or_divide_or_modulo} + {multiply_or_divide_or_modulo} - {multiply_or_divide_or_modulo}"
        );
        let expected_add_or_subtract = format!(
            "(({expected_multiply_or_divide_or_modulo} + {expected_multiply_or_divide_or_modulo}) - {expected_multiply_or_divide_or_modulo})"
        );

        let shift = format!("{add_or_subtract} << {add_or_subtract} >> {add_or_subtract}");
        let expected_shift = format!(
            "(({expected_add_or_subtract} << {expected_add_or_subtract}) >> {expected_add_or_subtract})"
        );

        let less_or_greater = format!("{shift} < {shift} > {shift} <= {shift} >= {shift}");
        let expected_less_or_greater = format!(
            "(((({expected_shift} < {expected_shift}) > {expected_shift}) <= {expected_shift}) >= {expected_shift})"
        );

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
    fn errors_on_logical_and() {
        let src = "
        1 && 2
          ^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expression = parser.parse_expression_or_error();
        assert_eq!(expression.to_string(), "(1 & 2)");

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::LogicalAnd));
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
        assert!(lambda.return_type.is_none());
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
        assert!(typ.is_none());

        let (pattern, typ) = lambda.parameters.remove(0);
        assert_eq!(pattern.to_string(), "y");
        assert_eq!(typ.unwrap().to_string(), "Field");
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
        assert_eq!(lambda.return_type.unwrap().to_string(), "Field");
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
    fn parses_type_path_with_tuple() {
        let src = "<()>::foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::TypePath(type_path) = expr.kind else {
            panic!("Expected type_path");
        };
        assert_eq!(type_path.typ.to_string(), "()");
        assert_eq!(type_path.item.to_string(), "foo");
        assert!(type_path.turbofish.is_none());
    }

    #[test]
    fn parses_type_path_with_array_type() {
        let src = "<[i32; 3]>::foo";
        let expr = parse_expression_no_errors(src);
        let ExpressionKind::TypePath(type_path) = expr.kind else {
            panic!("Expected type_path");
        };
        assert_eq!(type_path.typ.to_string(), "[i32; 3]");
        assert_eq!(type_path.item.to_string(), "foo");
        assert!(type_path.turbofish.is_none());
    }

    #[test]
    fn parses_type_path_with_empty_tuple_missing_angle_brackets() {
        let src = "
          ()::foo
          ^^
          ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        let ExpressionKind::TypePath(type_path) = expr.kind else {
            panic!("Expected type_path");
        };
        assert_eq!(type_path.typ.to_string(), "()");
        assert_eq!(type_path.item.to_string(), "foo");
        assert!(type_path.turbofish.is_none());

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::MissingAngleBrackets));
    }

    #[test]
    fn parses_type_path_with_non_empty_tuple_missing_angle_brackets() {
        let src = "
          (Field, i32)::foo
          ^^^^^^^^^^^^
          ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        assert!(matches!(expr.kind, ExpressionKind::Error));

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::MissingAngleBrackets));
    }

    #[test]
    fn parses_type_path_with_array_missing_angle_brackets() {
        let src = "
          [Field; 3]::foo
          ^^^^^^^^^^
          ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expr = parser.parse_expression_or_error();

        assert!(matches!(expr.kind, ExpressionKind::Error));

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::MissingAngleBrackets));
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

    #[test]
    fn parses_assert() {
        let src = "assert(true, \"good\")";
        let expression = parse_expression_no_errors(src);
        let ExpressionKind::Constrain(constrain) = expression.kind else {
            panic!("Expected constrain expression");
        };
        assert_eq!(constrain.kind, ConstrainKind::Assert);
        assert_eq!(constrain.arguments.len(), 2);
    }

    #[test]
    fn parses_assert_eq() {
        let src = "assert_eq(1, 2, \"bad\")";
        let expression = parse_expression_no_errors(src);
        let ExpressionKind::Constrain(constrain) = expression.kind else {
            panic!("Expected constrain expression");
        };
        assert_eq!(constrain.kind, ConstrainKind::AssertEq);
        assert_eq!(constrain.arguments.len(), 3);
    }

    #[test]
    fn parses_constrain() {
        let src = "
        constrain 1
        ^^^^^^^^^
        ";
        let (src, span) = get_source_with_error_span(src);
        let mut parser = Parser::for_str_with_dummy_file(&src);
        let expression = parser.parse_expression_or_error();
        let ExpressionKind::Constrain(constrain) = expression.kind else {
            panic!("Expected constrain expression");
        };
        assert_eq!(constrain.kind, ConstrainKind::Constrain);
        assert_eq!(constrain.arguments.len(), 1);

        let reason = get_single_error_reason(&parser.errors, span);
        assert!(matches!(reason, ParserErrorReason::ConstrainDeprecated));
    }

    #[test]
    fn errors_on_match_without_left_brace_after_expression() {
        let src = "
        fn main()  {
            if true {
                match c _ => {
                    match d _ => 0,                     
                }
            }
        } } } 
        ";
        let (_, errors) = parse_program_with_dummy_file(src);
        assert!(!errors.is_empty());
    }
}
