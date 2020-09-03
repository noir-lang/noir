use crate::{Precedence, Program};
use libnoirc_ast::{BlockStatement, Expression, ExpressionStatement, Statement};
use libnoirc_lexer::lexer::Lexer;
use libnoirc_lexer::token::{Attribute, Keyword, Token, TokenKind};
use std::error::Error;

type PrefixFn = fn(parser: &mut Parser) -> Expression;
type InfixFn = fn(parser: &mut Parser, left: Expression) -> Expression;

// XXX: We can probably abstract the lexer away, as we really only need an Iterator of Tokens/ TokenStream
// XXX: Alternatively can make Lexer take a Reader, but will need to do a Bytes -> to char conversion. Can just return an error if cannot do conversion
// As this should not be leaked to any other part of the lib
pub struct Parser<'a> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) curr_token: Token,
    pub(crate) peek_token: Token,
    pub(crate) errors: Vec<Box<Error>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let curr_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            curr_token,
            peek_token,
            errors: Vec::new(),
        }
    }
    pub(crate) fn advance_tokens(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
    // peaks at the next token
    // asserts that it should be of a certain variant
    // If it is, the parser is advanced
    pub(crate) fn peek_check_variant_advance(&mut self, token: Token) -> bool {
        let same_variant = self.peek_token.is_variant(&token);

        if same_variant {
            self.advance_tokens();
            return true;
        }
        return false;
    }
    // peaks at the next token
    // asserts that it should be of a certain kind
    // If it is, the parser is advanced
    pub(crate) fn peek_check_kind_advance(&mut self, token_kind: TokenKind) -> bool {
        let same_kind = self.peek_token.kind() == token_kind;
        if same_kind {
            self.advance_tokens();
            return true;
        }
        return false;
    }

    pub fn parse_program(&mut self) -> Program {
        use super::prefix_parser::FuncParser;

        let mut program = Program::with_capacity(self.lexer.by_ref().approx_len());

        while self.curr_token != Token::EOF {
            // First check if we have a function definition.
            // Function definitions are not added to the AST
            // Although we can have function literals starting with the function keyword
            // they will be self-contained within another function and they will start with a `let` token
            // Eg let add = fn(x,y) {x+y}
            match self.curr_token.clone() {
                Token::Keyword(Keyword::Fn) => {
                    let func_def = FuncParser::parse_fn_decl(self);
                    program.push_constraint_function(func_def);
                }
                _ => {
                    // Parse regular statements
                    let statement = self.parse_statement();
                    program.push_statement(statement);
                }
            }
            self.advance_tokens();
        }

        program
    }
    pub fn parse_statement(&mut self) -> Statement {
        use crate::constraint_parser::ConstraintParser;
        use crate::prefix_parser::DeclarationParser;

        // The first type of statement we could have is a variable declaration statement
        if self.curr_token.can_start_declaration() {
            return DeclarationParser::parse_declaration_statement(self, &self.curr_token.clone());
        };

        let stmt = match self.curr_token {
            Token::Keyword(Keyword::Constrain) => {
                Statement::Constrain(ConstraintParser::parse_constrain_statement(self))
            }
            _ => {
                let expr_stmt = self.parse_expression_statement();
                Statement::Expression(Box::new(expr_stmt))
            }
        };
        // Check if the next token is a semi-colon(optional)
        if self.peek_token == Token::Semicolon {
            self.advance_tokens();
        };
        return stmt;
    }

    fn parse_expression_statement(&mut self) -> ExpressionStatement {
        let expr = self.parse_expression(Precedence::Lowest);
        ExpressionStatement(expr.unwrap())
    }

    pub(crate) fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        // Calling this method means that we are at the beginning of a local expression
        // We may be in the middle of a global expression, but this does not matter
        let mut left_exp = match self.prefix_fn() {
            Some(prefix) => prefix(self),
            None => {
                println!("ERROR: No prefix function found for {}", &self.curr_token);
                return None;
            }
        };
        while (self.peek_token != Token::Semicolon)
            && (precedence < Precedence::from(&self.peek_token))
        {
            match self.infix_fn() {
                None => {
                    println!("No infix function found for {}", &self.curr_token); // XXX: This is a user error, so that's why we don't panic. Unless I forgot to implement an infix for a operator
                    return Some(left_exp.clone());
                }
                Some(infix_fn) => {
                    self.advance_tokens();
                    left_exp = infix_fn(self, left_exp);
                }
            }
        }

        return Some(left_exp);
    }
    fn prefix_fn(&self) -> Option<PrefixFn> {
        use crate::prefix_parser::{GroupParser, IfParser, LiteralParser, NameParser, UnaryParser};
        use crate::PrefixParser;

        match &self.curr_token {
            Token::Keyword(Keyword::If) => Some(IfParser::parse),
            x if x.kind() == TokenKind::Ident => Some(NameParser::parse),
            x if x.kind() == TokenKind::Literal => Some(LiteralParser::parse),
            Token::Bang | Token::Minus => Some(UnaryParser::parse),
            Token::LeftParen => Some(GroupParser::parse),
            _ => None,
        }
    }
    fn infix_fn(&mut self) -> Option<InfixFn> {
        use crate::infix_parser::{BinaryParser, CallParser};
        use crate::InfixParser;

        match self.peek_token {
            Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Pipe
            | Token::Ampersand
            | Token::Star
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Equal
            | Token::NotEqual => Some(BinaryParser::parse),
            Token::LeftParen => Some(CallParser::parse),
            _ => None,
        }
    }

    pub(crate) fn parse_block_statement(&mut self) -> BlockStatement {
        let mut statements: Vec<Statement> = Vec::new();

        self.advance_tokens();

        while (self.curr_token != Token::RightBrace) && (self.curr_token != Token::EOF) {
            statements.push(self.parse_statement());
            self.advance_tokens();
        }

        BlockStatement(statements)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use libnoirc_ast::{
        BlockStatement, CallExpression, Expression, ExpressionStatement, FunctionDefinition,
        FunctionLiteral, Ident, IfExpression, InfixExpression, Literal, PrefixExpression,
        Statement, Type,
    };
    #[test]
    fn test_basic_let() {
        // XXX: Incomplete, as we do not check the expression
        let input = "
    
    let x = 5;
    let y = 15;
    let z = 20;
    ";

        let test_iden = vec!["x", "y", "z"];

        let mut parser = Parser::new(Lexer::new(input));

        let program = parser.parse_program();
        for (stmt, iden) in program.statements.iter().zip(test_iden.iter()) {
            helper_test_let(stmt, iden);
        }

        assert_eq!(program.statements.len(), 3);
    }

    fn helper_test_let(statement: &Statement, iden: &str) {
        // First make sure that the statement is a let statement
        let let_stmt = match statement {
            Statement::Let(stmt) => stmt,
            _ => panic!("Expected a let statement"),
        };

        // Now assert the correct identifier is in the let statement
        assert_eq!(let_stmt.identifier.0, iden);
    }

    #[test]
    fn test_parse_identifier() {
        let input = "hello;world;This_is_a_word";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let test_iden = vec!["hello", "world", "This_is_a_word"];

        for (stmt, iden) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x.0,
                _ => unreachable!(),
            };
            // Extract the identifier
            let name = match expression {
                Expression::Ident(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(iden, &name)
        }
    }

    #[test]
    fn test_parse_literals() {
        let input = "10;true;\"string_literal\"";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let test_iden = vec![
            Literal::Integer(10),
            Literal::Bool(true),
            Literal::Str("string_literal".to_string()),
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x.0,
                _ => unreachable!(),
            };
            // Extract the literal
            let literal = match expression {
                Expression::Literal(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(expected_lit, &literal)
        }
    }
    #[test]
    fn test_parse_prefix() {
        use libnoirc_ast::*;
        let input = "!99;-100;!true";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let test_iden = vec![
            PrefixExpression {
                operator: UnaryOp::Not,
                rhs: Expression::Literal(Literal::Integer(99)),
            },
            PrefixExpression {
                operator: UnaryOp::Minus,
                rhs: Expression::Literal(Literal::Integer(100)),
            },
            PrefixExpression {
                operator: UnaryOp::Not,
                rhs: Expression::Literal(Literal::Bool(true)),
            },
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x.0,
                _ => unreachable!(),
            };
            // Extract the prefix expression
            let literal = match expression {
                Expression::Prefix(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *literal)
        }
    }

    #[test]
    fn test_parse_infix() {
        let input = "5+5;10*5;true == false; false != false";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let test_iden = vec![
            InfixExpression {
                lhs: Expression::Literal(Literal::Integer(5)),
                operator: Token::Plus.into(),
                rhs: Expression::Literal(Literal::Integer(5)),
            },
            InfixExpression {
                lhs: Expression::Literal(Literal::Integer(10)),
                operator: Token::Star.into(),
                rhs: Expression::Literal(Literal::Integer(5)),
            },
            InfixExpression {
                lhs: Expression::Literal(Literal::Bool(true)),
                operator: Token::Equal.into(),
                rhs: Expression::Literal(Literal::Bool(false)),
            },
            InfixExpression {
                lhs: Expression::Literal(Literal::Bool(false)),
                operator: Token::NotEqual.into(),
                rhs: Expression::Literal(Literal::Bool(false)),
            },
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x.0,
                _ => unreachable!(),
            };
            // Extract the infix expression
            let literal = match expression {
                Expression::Predicate(x) => x,
                Expression::Infix(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *literal)
        }
    }
    #[test]
    fn test_parse_grouped() {
        use libnoirc_ast::UnaryOp;

        let input = "-(5+10);-5+10";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        // Test the first expression : -(5+10)
        let grouped_expression = PrefixExpression {
            operator: UnaryOp::Minus,
            rhs: Expression::Infix(Box::new(InfixExpression {
                lhs: Expression::Literal(Literal::Integer(5)),
                operator: Token::Plus.into(),
                rhs: Expression::Literal(Literal::Integer(10)),
            })),
        };

        let stmt = program.statements[0].clone();
        let expected_lit = grouped_expression;
        // Cast to an expression
        let expression = match stmt {
            Statement::Expression(x) => x.0,
            _ => unreachable!(),
        };
        // Extract the prefix expression
        let prefix = match expression {
            Expression::Prefix(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*prefix, expected_lit);

        // Test the second expression : -5+10
        let ungrouped_expression = InfixExpression {
            lhs: Expression::Prefix(Box::new(PrefixExpression {
                operator: UnaryOp::Minus,
                rhs: Expression::Literal(Literal::Integer(5)),
            })),
            operator: Token::Plus.into(),
            rhs: Expression::Literal(Literal::Integer(10)),
        };

        let stmt = program.statements[1].clone();
        let expected_lit = ungrouped_expression;
        // Cast to an expression
        let expression = match stmt {
            Statement::Expression(x) => x.0,
            _ => unreachable!(),
        };
        // Extract the prefix expression
        let prefix = match expression {
            Expression::Infix(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*prefix, expected_lit);
    }

    #[test]
    fn test_parse_if_expression() {
        let input = "if (x < y) { x };";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let expected_if = IfExpression {
            condition: Expression::Predicate(Box::new(InfixExpression {
                lhs: Expression::Ident("x".to_string()),
                operator: Token::Less.into(),
                rhs: Expression::Ident("y".to_string()),
            })),
            consequence: BlockStatement(vec![Statement::Expression(Box::new(
                ExpressionStatement(Expression::Ident("x".to_string())),
            ))]),
            alternative: None,
        };

        let stmt = program.statements[0].clone();
        let expression = match stmt {
            Statement::Expression(x) => x.0,
            _ => unreachable!(),
        };
        // Extract the if expression
        let if_expr = match expression {
            Expression::If(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*if_expr, expected_if)
    }
    #[test]
    fn test_parse_if_else_expression() {
        let input = "if (foo < bar) { cat } else {dog};";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let expected_if = IfExpression {
            condition: Expression::Predicate(Box::new(InfixExpression {
                lhs: Expression::Ident("foo".to_string()),
                operator: Token::Less.into(),
                rhs: Expression::Ident("bar".to_string()),
            })),
            consequence: BlockStatement(vec![Statement::Expression(Box::new(
                ExpressionStatement(Expression::Ident("cat".to_string())),
            ))]),
            alternative: Some(BlockStatement(vec![Statement::Expression(Box::new(
                ExpressionStatement(Expression::Ident("dog".to_string())),
            ))])),
        };

        let stmt = program.statements[0].clone();
        let expression = match stmt {
            Statement::Expression(x) => x.0,
            _ => unreachable!(),
        };
        // Extract the if expression
        let if_expr = match expression {
            Expression::If(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*if_expr, expected_if)
    }

    // XXX: Lets move this test into the func lit parser module
    #[test]
    fn test_parse_function_literal() {
        use crate::prefix_parser::FuncParser;
        let input = "fn(x : Witness,y : Witness){x+y;}";
        let mut parser = Parser::new(Lexer::new(input));
        let (func_dec, func_lit) = FuncParser::parse_fn(&mut parser);

        assert!(func_dec.is_none());
        assert!(func_lit.is_some());
        let func_lit = func_lit.unwrap();

        let parameters = vec![
            (Ident("x".into()), Type::Witness),
            (Ident("y".into()), Type::Witness),
        ];

        let infix_expression = InfixExpression {
            lhs: Expression::Ident("x".to_string()),
            operator: Token::Plus.into(),
            rhs: Expression::Ident("y".to_string()),
        };

        let expected = FunctionLiteral {
            parameters: parameters,
            body: BlockStatement(vec![Statement::Expression(Box::new(ExpressionStatement(
                Expression::Infix(Box::new(infix_expression)),
            )))]),
        };
        assert_eq!(func_lit, expected);
    }

    #[test]
    // XXX: This just duplicates most of test_funct_literal. Refactor to avoid duplicate code
    fn test_parse_function_def_literal() {
        let input = "fn add(x : Public,y : Constant){x+y}";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let parameters = vec![
            (Ident("x".into()), Type::Public),
            (Ident("y".into()), Type::Constant),
        ];

        let infix_expression = InfixExpression {
            lhs: Expression::Ident("x".to_string()),
            operator: Token::Plus.into(),
            rhs: Expression::Ident("y".to_string()),
        };

        let func = FunctionLiteral {
            parameters: parameters,
            body: BlockStatement(vec![Statement::Expression(Box::new(ExpressionStatement(
                Expression::Infix(Box::new(infix_expression)),
            )))]),
        };

        let expected = vec![FunctionDefinition {
            name: Ident("add".into()),
            func,
        }];

        for (expected_def, got_def) in expected.into_iter().zip(program.functions.into_iter()) {
            assert_eq!(expected_def, got_def);
        }
    }

    #[test]
    fn test_parse_call_expression() {
        let input = "add(1,2+3)";
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program();

        let test_iden = vec![CallExpression {
            func_name: Ident("add".to_string()),
            arguments: vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Infix(Box::new(InfixExpression {
                    lhs: Expression::Literal(Literal::Integer(2)),
                    operator: Token::Plus.into(),
                    rhs: Expression::Literal(Literal::Integer(3)),
                })),
            ],
        }];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x.0,
                _ => unreachable!(),
            };
            // Extract the function literal expression
            let call_expr = match expression {
                Expression::Call(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *call_expr)
        }
    }
}
