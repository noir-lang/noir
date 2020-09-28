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

pub(crate) fn parse_let_statement(parser: &mut Parser) -> Box<LetStatement> {
    let (name, typ, expr) = parse_generic_decl_statement(parser);

    let stmt = LetStatement {
        identifier: name,
        r#type: typ.unwrap_or(Type::Error), //XXX: Haven't implemented this yet for general structs, we only parse arrays using this
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

pub(crate) fn parse_decl_type(parser: &mut Parser) -> Type {
    // Currently we only support the default types and integers.
    // If we get into this function, then the user is specifying a type
    match &parser.curr_token {
        Token::Keyword(Keyword::Witness) => Type::Witness,
        Token::Keyword(Keyword::Public) => Type::Public,
        Token::IntType(int_type) => int_type.into(),
        Token::LeftBracket => parse_array_type(parser),
        k => unimplemented!("This type is currently not supported, {}", k),
    }
}

fn parse_array_type(parser: &mut Parser) -> Type {
    // Expression is of the form [3]Type

    // Current token is '['
    //
    // Next token should be an Integer
    if !parser.peek_check_int() {
        panic!("Expected an Int")
    };
    let array_len = match parser.curr_token {
        Token::Int(integer) => integer,
        _ => panic!("User error: Expected an Integer for the array length"),
    };

    if array_len < 0 {
        panic!("Cannot have a negative array size, [-k]Type is disallowed")
    }
    let array_len = array_len as u128;

    if !parser.peek_check_variant_advance(Token::RightBracket) {
        panic!(
            "expected a right bracket after integer, got {}",
            parser.peek_token
        )
    }

    // Skip Right bracket
    parser.advance_tokens();

    // Disallow [4][3]Witness ie Matrices
    if parser.peek_token == Token::LeftBracket {
        panic!("Currently Multi-dimensional arrays are not supported")
    }

    let array_type = parse_decl_type(parser);

    Type::Array(array_len, Box::new(array_type))
}
