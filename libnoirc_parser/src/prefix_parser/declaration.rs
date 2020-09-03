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

fn parse_generic_decl_statement(parser: &mut Parser) -> (Ident, Expression) {
    // Expect an identifier
    assert!(parser.peek_token.kind() == TokenKind::Ident);
    if !parser.peek_check_kind_advance(TokenKind::Ident) {
        panic!("expected an identifier");
    };

    let name = parser.curr_token.to_string();

    // Expect an assign
    if !parser.peek_check_variant_advance(Token::Assign) {
        panic!("expected an assign token")
    };

    parser.advance_tokens(); // Skip the assign

    let expr = parser.parse_expression(Precedence::Lowest).unwrap();

    if parser.peek_token == Token::Semicolon {
        parser.advance_tokens();
    }

    (name.into(), expr)
}

pub(crate) fn parse_let_statement(parser: &mut Parser) -> Box<LetStatement> {
    let (name, expr) = parse_generic_decl_statement(parser);
    let stmt = LetStatement {
        identifier: name,
        r#type: Type::Error,
        expression: expr,
    };
    Box::new(stmt)
}
pub(crate) fn parse_const_statement(parser: &mut Parser) -> Box<ConstStatement> {
    let (name, expr) = parse_generic_decl_statement(parser);
    let stmt = ConstStatement {
        identifier: name,
        r#type: Type::FieldElement,
        expression: expr,
    };
    Box::new(stmt)
}
pub(crate) fn parse_private_statement(parser: &mut Parser) -> Box<PrivateStatement> {
    let (name, expr) = parse_generic_decl_statement(parser);
    let stmt = PrivateStatement {
        identifier: name,
        r#type: Type::Witness,
        expression: expr,
    };
    Box::new(stmt)
}
pub(crate) fn parse_public_statement(parser: &mut Parser) -> Box<PublicStatement> {
    let (name, expr) = parse_generic_decl_statement(parser);
    let stmt = PublicStatement {
        identifier: name,
        r#type: Type::FieldElement,
        expression: expr,
    };
    Box::new(stmt)
}
