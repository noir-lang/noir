use super::*;

/// `let` , `priv`, `const` and `pub` are used for variable declaration
///
/// Since they follow the same structure, the parsing strategy for
/// them are generic
struct GenericDeclStructure {
    identifier: Ident,
    typ: Option<Type>,
    rhs: Expression,
}

pub struct DeclarationParser;

impl DeclarationParser {
    /// Parses a generic declaration statement. The token parameter determines what type of statement will be parsed
    ///
    /// XXX: We can make this clearer by further separating keywords into declaration keywords.
    /// This would ensure that the match statement will always be exhaustive on the list
    /// of declaration keywords.
    pub fn parse_statement(parser: &mut Parser) -> Result<Statement, ParserErrorKind> {
        match parser.curr_token.token().to_declaration_keyword() {
            Keyword::Let => Ok(Statement::Let(parse_let_statement(parser)?)),
            Keyword::Const => Ok(Statement::Const(parse_const_statement(parser)?)),
            Keyword::Priv => Ok(Statement::Private(parse_private_statement(parser)?)),
            kw => {
                let msg = format!("the keyword {} cannot be used to declare a statement. Please use `let`, `const` or `priv`", kw);
                return Err(ParserErrorKind::UnstructuredError {
                    span: parser.curr_token.into_span(),
                    message: msg,
                });
            }
        }
    }
}

/// Parses statements of the form
/// - DECL_KEYWORD IDENT : TYPE? = EXPR
///
/// The TYPE? signifies that the parameter can be optional
///
/// Cursor Start : `DECL_KEYWORD`
///
/// Cursor End : `;`
fn parse_generic_decl_statement(
    parser: &mut Parser,
) -> Result<GenericDeclStructure, ParserErrorKind> {
    // The next token should be an identifier
    parser.peek_check_kind_advance(TokenKind::Ident)?;

    let identifier: Ident = parser.curr_token.clone().into();

    let mut declared_typ = None;
    if parser.peek_token == Token::Colon {
        // Advance past the identifier.
        // Current token is now the Colon
        parser.advance_tokens();
        // Advance past the Colon
        // Current token is now the Type
        parser.advance_tokens(); // Advance to the type token

        declared_typ = Some(parser.parse_type(true)?);
    };

    // Current token is the Type
    // Expect an `=` token. Bump
    parser.peek_check_variant_advance(&Token::Assign)?;

    // Current token is now an `=` .
    // Bump cursor to the expression.
    parser.advance_tokens();

    // Cursor now points to the last token in the expression
    let expr = parser.parse_expression(Precedence::Lowest)?;

    // Since these are statements, they must end in a semi colon.
    // XXX: Add a `help` note to tell the user to add a semi colon here
    parser.peek_check_variant_advance(&Token::Semicolon)?;

    Ok(GenericDeclStructure {
        identifier,
        typ: declared_typ,
        rhs: expr,
    })
}

fn parse_let_statement(parser: &mut Parser) -> Result<LetStatement, ParserErrorKind> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    let stmt = LetStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified), //XXX: Haven't implemented this yet for general structs, we only parse arrays using this
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
fn parse_const_statement(parser: &mut Parser) -> Result<ConstStatement, ParserErrorKind> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // Note: If a Type is supplied for some reason in a const statement, it can only be a Field element/Constant
    let mut default_type = Type::CONSTANT;
    if let Some(declared_type) = generic_stmt.typ {
        if !declared_type.is_constant() {
            let message = format!("Const statements can only have constant type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Constant ",declared_type);
            return Err(ParserErrorKind::UnstructuredError {
                message,
                span: Span::default(),
            }); // XXX: We don't have spanning for types yet
        } else {
            default_type = declared_type;
        }
    }

    let stmt = ConstStatement {
        identifier: generic_stmt.identifier,
        r#type: default_type,
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
fn parse_private_statement(parser: &mut Parser) -> Result<PrivateStatement, ParserErrorKind> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // XXX: As of FieldElement refactor, we can catch basic type errors for private statements,
    // similar to the code for pub and const.
    //
    // This change may wait until, it is decided whether `let` will be the only declaration keyword.

    let stmt = PrivateStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified),
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}

//XXX: Maybe do a second pass for invalid?
#[cfg(test)]
mod test {
    use crate::{parser::test_parse, token::Token};

    use super::DeclarationParser;

    #[test]
    fn valid_let_syntax() {
        /// Why is it valid to specify a let declaration as having type u8?
        ///
        /// Let statements are not type checked here, so the parser will accept as
        /// long as it is a type. Other statements such as Public are type checked
        /// Because for now, they can only have one type
        const VALID: &'static [&str] = &[
            r#"
                let x = y;
            "#,
            r#"
                let x : u8 = y;
            "#,
        ];

        for valid in VALID {
            let mut parser = test_parse(valid);
            let start = parser.curr_token.clone();
            DeclarationParser::parse_statement(&mut parser).unwrap();
            let end = parser.curr_token.clone();

            // First check that the cursor was in the right position at
            // the start and at the end
            assert_eq!(start.token(), &Token::Keyword(crate::token::Keyword::Let));
            assert_eq!(end.token(), &Token::Semicolon);
        }
    }
    #[test]
    fn valid_priv_syntax() {
        /// Private statements are also not checked here on parsing
        /// as there are many private types.
        ///
        /// It is possible to check for basic types in the future.
        const VALID: &'static [&str] = &[
            r#"
                priv x = y;
            "#,
            r#"
                priv x : pub Field = y;
            "#,
        ];

        for valid in VALID {
            let mut parser = test_parse(valid);
            let start = parser.curr_token.clone();
            DeclarationParser::parse_statement(&mut parser).unwrap();
            let end = parser.curr_token.clone();

            // First check that the cursor was in the right position at
            // the start and at the end
            assert_eq!(start.token(), &Token::Keyword(crate::token::Keyword::Priv));
            assert_eq!(end.token(), &Token::Semicolon);
        }
    }
    #[test]
    fn invalid_pub_syntax() {
        // pub cannot be used to declare a statement
        const INVALID: &'static [&str] = &[
            r#"
                pub x = y;
            "#,
            r#"
                pub x : pub Field = y;
            "#,
        ];

        for invalid in INVALID {
            let mut parser = test_parse(invalid);
            assert!(DeclarationParser::parse_statement(&mut parser).is_err());
        }
    }
    #[test]
    fn valid_const_syntax() {
        /// XXX: We have `Constant` because we may allow constants to
        /// be casted to integers. Maybe rename this to `Field` instead
        const VALID: &'static [&str] = &[
            r#"
                const x = y;
            "#,
            r#"
                const x : const Field = y;
            "#,
        ];

        for valid in VALID {
            let mut parser = test_parse(valid);
            let start = parser.curr_token.clone();
            DeclarationParser::parse_statement(&mut parser).unwrap();
            let end = parser.curr_token.clone();

            // First check that the cursor was in the right position at
            // the start and at the end
            assert_eq!(start.token(), &Token::Keyword(crate::token::Keyword::Const));
            assert_eq!(end.token(), &Token::Semicolon);
        }
    }
}
