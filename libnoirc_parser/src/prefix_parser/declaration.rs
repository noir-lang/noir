use super::*;

// Currently the following keywords
// lets us know that we are declaring a variable 'let', 'priv','const', 'pub'
// They follow the same structure

pub struct DeclarationParser;

impl DeclarationParser {
    /// Parses a declaration statement. The token parameter determines what type of statement will be parsed
    pub fn parse_declaration_statement(parser: &mut Parser, token: &Token) -> Statement {
        match token.to_declaration_keyword() {
            Keyword::Let=> {
                Statement::Let(parse_let_statement(parser))
            },
             Keyword::Const =>{
                 Statement::Const(parse_const_statement(parser))
             },
            Keyword::Pub=>{
                  Statement::Public(parse_public_statement(parser))
            }, 
            Keyword::Private => {
                  Statement::Private(parse_private_statement(parser))
            }
            kw=> panic!("Bug : All declaration keywords should have a method to parse their structure, the keyword {} does not", kw)
        }
    }
}

fn parse_generic_decl_statement(parser: &mut Parser) -> (Ident, Option<Type>, Expression) {
    // Expect an identifier
    assert!(parser.peek_token.kind() == TokenKind::Ident);
    if !parser.peek_check_kind_advance(TokenKind::Ident) {
        panic!("expected an identifier");
    };
    let name = parser.curr_token.to_string();

    let mut typ = None;

    // Check for colon
    if parser.peek_check_variant_advance(Token::Colon) {
        parser.advance_tokens();

        // Parse the type
        typ = Some(parse_decl_type(parser));
    };

    // Expect an assign
    if !parser.peek_check_variant_advance(Token::Assign) {
        panic!("expected an assign token")
    };

    parser.advance_tokens(); // Skip the assign

    let expr = parser.parse_expression(Precedence::Lowest).unwrap();

    if parser.peek_token == Token::Semicolon {
        parser.advance_tokens();
    }

    (name.into(), typ, expr)
}

pub(crate) fn parse_decl_type(parser: &mut Parser) -> Type {
    // Currently we only support the default types and integers.
    // If we get into this function, then the user is specifying a type
    match &parser.curr_token {
        Token::Keyword(Keyword::Witness) => Type::Witness,
        Token::Keyword(Keyword::Public) => Type::Public,
        Token::IntType(int_type) => int_type.into(),
        _ => unimplemented!("This type is currently not supported"),
    }
}
pub(crate) fn parse_let_statement(parser: &mut Parser) -> Box<LetStatement> {
    let (name, typ, expr) = parse_generic_decl_statement(parser);
    let stmt = LetStatement {
        identifier: name,
        r#type: Type::Error, //XXX: Haven't implemented this yet for general structs
        expression: expr,
    };
    Box::new(stmt)
}
pub(crate) fn parse_const_statement(parser: &mut Parser) -> Box<ConstStatement> {
    let (name, typ, expr) = parse_generic_decl_statement(parser);

    // Note: If a Type is supplied for some reason in a const statement, it can only be a Field element
    match typ {
        Some(declared_typ) => assert_eq!(declared_typ , Type::FieldElement, "Const statements can only have field elements type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Field ",declared_typ),
        None => {}
    };

    let stmt = ConstStatement {
        identifier: name,
        r#type: Type::FieldElement,
        expression: expr,
    };
    Box::new(stmt)
}
pub(crate) fn parse_private_statement(parser: &mut Parser) -> Box<PrivateStatement> {
    let (name, typ, expr) = parse_generic_decl_statement(parser);

    let stmt = PrivateStatement {
        identifier: name,
        r#type: typ.unwrap_or(Type::Witness),
        expression: expr,
    };
    Box::new(stmt)
}
// XXX: We most likely will deprecate a Public statement, as users will not be able to
pub(crate) fn parse_public_statement(parser: &mut Parser) -> Box<PublicStatement> {
    let (name, typ, expr) = parse_generic_decl_statement(parser);

    // Note: If a Type is supplied for some reason in a const statement, it can only be Public for now.
    //XXX: Still TBD, if we will remove public statements, and only allow public inputs to be supplied via main
    match typ {
            Some(declared_typ) => assert_eq!(declared_typ , Type::Public, "Public statements can only have public type, you supplied a {:?}. Suggestion: Remove the type and the compiler will default to Public ",declared_typ),
            None => {}
        };

    let stmt = PublicStatement {
        identifier: name,
        r#type: Type::Public,
        expression: expr,
    };
    Box::new(stmt)
}
