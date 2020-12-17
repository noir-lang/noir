use super::*;

// Currently the following keywords
// lets us know that we are declaring a variable 'let', 'priv','const', 'pub'
// They follow the same structure

struct GenericDeclStructure {
    identifier : Ident,
    typ: Option<Type>,
    rhs : Expression,
}


pub struct DeclarationParser;

impl DeclarationParser {
    /// Parses a declaration statement. The token parameter determines what type of statement will be parsed
    /// XXX: We can make this better,  by separating the keyword variants from the declaration variants(embedded)
    pub fn parse_declaration_statement(parser: &mut Parser) -> Result<Statement, ParserError> {
        match parser.curr_token.token().to_declaration_keyword() {
            Keyword::Let=> {
                Ok(Statement::Let(parse_let_statement(parser)?))
            },
             Keyword::Const =>{
                 Ok(Statement::Const(parse_const_statement(parser)?))
             },
            Keyword::Pub=>{
                  Ok(Statement::Public(parse_public_statement(parser)?))
            }, 
            Keyword::Private => {
                  Ok(Statement::Private(parse_private_statement(parser)?))
            }
            kw => {
                let message = format!("All declaration keywords should have a method to parser their structure, the keyword {} does not have this", kw);
                return Err(ParserErrorKind::InternalError{message, span : parser.curr_token.into_span() }.into_err(parser.file_id));
            }
        }
    }
}

fn parse_generic_decl_statement(parser: &mut Parser) -> Result<GenericDeclStructure, ParserError> {
    // Expect an identifier
    parser.peek_check_kind_advance(TokenKind::Ident)?;
    let spanned_name : Ident = parser.curr_token.clone().into();
    
    let mut typ = None;
    
    // Check for colon
    if parser.peek_token == Token::Colon {
        parser.advance_tokens();
        parser.advance_tokens();
        
        // Parse the type
        typ = Some(parser.parse_type()?);
    };
    
    // Expect an assign
    parser.peek_check_variant_advance(&Token::Assign)?;

    parser.advance_tokens(); // Skip the assign

    let expr = parser.parse_expression(Precedence::Lowest)?;

    if parser.peek_token == Token::Semicolon {
        parser.advance_tokens();
    }

    Ok(GenericDeclStructure {
        identifier : spanned_name,
        typ,
        rhs : expr
    })
}

pub(crate) fn parse_let_statement(parser: &mut Parser) -> Result<LetStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    let stmt = LetStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified), //XXX: Haven't implemented this yet for general structs, we only parse arrays using this
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
pub(crate) fn parse_const_statement(parser: &mut Parser) -> Result<ConstStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // Note: If a Type is supplied for some reason in a const statement, it can only be a Field element/Constant
    if let Some(declared_type) = generic_stmt.typ {
        if declared_type != Type::Constant {
            let message = format!("Const statements can only have constant type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Constant ",declared_type);
            return Err(ParserErrorKind::UnstructuredError{message, span : Span::default()}.into_err(parser.file_id)) // XXX: We don't have spanning for types yet
        }
    }
    
    let stmt = ConstStatement {
        identifier: generic_stmt.identifier,
        r#type: Type::Constant,
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
pub(crate) fn parse_private_statement(parser: &mut Parser) -> Result<PrivateStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    let stmt = PrivateStatement {
        identifier: generic_stmt.identifier,
        r#type: generic_stmt.typ.unwrap_or(Type::Unspecified),
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
// XXX: We most likely will deprecate a Public statement, as users will not be able to
pub(crate) fn parse_public_statement(parser: &mut Parser) -> Result<PublicStatement, ParserError> {
    let generic_stmt = parse_generic_decl_statement(parser)?;

    // Note: If a Type is supplied for some reason in a const statement, it can only be Public for now.
    //XXX: Still TBD, if we will remove public statements, and only allow public inputs to be supplied via main
        if let Some(declared_type) = generic_stmt.typ {
            if declared_type != Type::Public {
                let message = format!("Public statements can only have public type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Public ",declared_type);
                return Err(ParserErrorKind::UnstructuredError{message, span : Span::default()}.into_err(parser.file_id)) // XXX: We don't have spanning for types yet
            }
        }

    let stmt = PublicStatement {
        identifier: generic_stmt.identifier,
        r#type: Type::Public,
        expression: generic_stmt.rhs,
    };
    Ok(stmt)
}
