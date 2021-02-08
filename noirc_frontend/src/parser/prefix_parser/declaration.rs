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
    pub fn parse_statement(parser: &mut Parser) -> Result<Statement, ParserError> {
        match parser.curr_token.token().to_declaration_keyword() {
            Keyword::Let => Ok(Statement::Let(parse_let_statement(parser)?)),
            Keyword::Const => Ok(Statement::Const(parse_const_statement(parser)?)),
            Keyword::Pub => Ok(Statement::Public(parse_public_statement(parser)?)),
            Keyword::Private => Ok(Statement::Private(parse_private_statement(parser)?)),
            kw => {
                let message = format!("All declaration keywords should have a method to parser their structure, the keyword {} does not have this", kw);
                return Err(ParserErrorKind::InternalError {
                    message,
                    span: parser.curr_token.into_span(),
                }
                .into_err(parser.file_id));
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
fn parse_generic_decl_statement(parser: &mut Parser) -> Result<GenericDeclStructure, ParserError> {
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

        declared_typ = Some(parser.parse_type()?);
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

fn parse_let_statement(parser: &mut Parser) -> Result<LetStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    let stmt = LetStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified), //XXX: Haven't implemented this yet for general structs, we only parse arrays using this
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
fn parse_const_statement(parser: &mut Parser) -> Result<ConstStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // Note: If a Type is supplied for some reason in a const statement, it can only be a Field element/Constant
    if let Some(declared_type) = generic_stmt.typ {
        if declared_type != Type::Constant {
            let message = format!("Const statements can only have constant type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Constant ",declared_type);
            return Err(ParserErrorKind::UnstructuredError {
                message,
                span: Span::default(),
            }
            .into_err(parser.file_id)); // XXX: We don't have spanning for types yet
        }
    }

    let stmt = ConstStatement {
        identifier: generic_stmt.identifier,
        r#type: Type::Constant,
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
fn parse_private_statement(parser: &mut Parser) -> Result<PrivateStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    let stmt = PrivateStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified),
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
// XXX : Currently Public statements, can only be specified in the ABi parameters.
//
// XXX: More research is required to determine whether this can be fully deprecated.
// In place for something like : `witness.to_public()`
fn parse_public_statement(parser: &mut Parser) -> Result<PublicStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // Note: If a Type is supplied for some reason in a public statement, it can only be Public for now.
    if let Some(declared_type) = generic_stmt.typ {
        if declared_type != Type::Public {
            let message = format!("Public statements can only have public type, you supplied a {}. Suggestion: Remove the type and the compiler will default to Public ",declared_type);
            // XXX: We don't have spanning for types yet
            return Err(ParserErrorKind::UnstructuredError {
                message,
                span: Span::default(),
            }
            .into_err(parser.file_id));
        }
    }

    let stmt = PublicStatement {
        identifier: generic_stmt.identifier,
        r#type: Type::Public,
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
                priv x : Public = y;
            "#,
        ];

        for valid in VALID {
            let mut parser = test_parse(valid);
            let start = parser.curr_token.clone();
            DeclarationParser::parse_statement(&mut parser).unwrap();
            let end = parser.curr_token.clone();

            // First check that the cursor was in the right position at
            // the start and at the end
            assert_eq!(
                start.token(),
                &Token::Keyword(crate::token::Keyword::Private)
            );
            assert_eq!(end.token(), &Token::Semicolon);
        }
    }
    #[test]
    fn valid_pub_syntax() {
        const VALID: &'static [&str] = &[
            r#"
                pub x = y;
            "#,
            r#"
                pub x : Public = y;
            "#,
        ];

        for valid in VALID {
            let mut parser = test_parse(valid);
            let start = parser.curr_token.clone();
            DeclarationParser::parse_statement(&mut parser).unwrap();
            let end = parser.curr_token.clone();

            // First check that the cursor was in the right position at
            // the start and at the end
            assert_eq!(start.token(), &Token::Keyword(crate::token::Keyword::Pub));
            assert_eq!(end.token(), &Token::Semicolon);
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
                const x : Constant = y;
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
