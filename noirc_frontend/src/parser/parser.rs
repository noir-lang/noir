use super::{Precedence, Program};
use crate::ast::{BlockStatement, Expression, Statement, Type, ArraySize, ExpressionKind};
use crate::lexer::Lexer;
use crate::token::{Keyword, Token, TokenKind, SpannedToken};
use super::errors::ParserError;

use super::prefix_parser::PrefixParser;
use super::infix_parser::InfixParser;

pub type ParserResult<T> = Result<T, ParserError>;
pub type ParserExprKindResult = ParserResult<ExpressionKind>;
pub type ParserExprResult = ParserResult<Expression>;
type ParserStmtResult = ParserResult<Statement>;

// XXX: We can probably abstract the lexer away, as we really only need an Iterator of Tokens/ TokenStream
// XXX: Alternatively can make Lexer take a Reader, but will need to do a Bytes -> to char conversion. Can just return an error if cannot do conversion
// As this should not be leaked to any other part of the lib
pub struct Parser<'a> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) curr_token: SpannedToken,
    pub(crate) peek_token: SpannedToken,
    pub(crate) errors: Vec<ParserError>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let curr_token = lexer.next_token().unwrap();
        let peek_token = lexer.next_token().unwrap();
        Parser {
            lexer,
            curr_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    pub fn with_input(input : &'a str) -> Self {
        Parser::new(Lexer::new(0,input))
    }

    /// Note that this function does not alert the user of an EOF
    /// calling this function repeatedly will repeatedly give you 
    /// an EOF token. EOF tokens are not errors
    pub(crate) fn advance_tokens(&mut self) {
        self.curr_token = self.peek_token.clone();

        loop {
            match self.lexer.next_token() {
                Ok(spanned_token) => {
                    self.peek_token = spanned_token;
                    break;
                },
                Err(lex_err) => {
                    self.errors.push(ParserError::LexerError(lex_err))
                }        
            }
        }

        // At this point, we could check for lexer errors
        // and abort, however, if we do not we may be able to 
        // recover if the next token is the correct one
    }
    // peaks at the next token
    // asserts that it should be of a certain variant
    // If it is, the parser is advanced
    pub(crate) fn peek_check_variant_advance(&mut self, token: &Token) -> Result<(), ParserError> {
        let same_variant = self.peek_token.is_variant(token);

        if !same_variant {
            let peeked_span = self.peek_token.into_span();
            let peeked_token = self.peek_token.token().clone();
            self.advance_tokens(); // We advance the token regardless, so the parser does not choke on a prefix function
            return Err(ParserError::UnexpectedToken{span : peeked_span, expected : token.clone(),found : peeked_token });
        }
        self.advance_tokens();
        return Ok(());
    }
    // peaks at the next token
    // asserts that it should be of a certain kind
    // If it is, the parser is advanced
    pub(crate) fn peek_check_kind_advance(&mut self, token_kind: TokenKind) -> Result<(), ParserError> {
        let peeked_kind = self.peek_token.kind();
        let same_kind = peeked_kind == token_kind;
        if !same_kind {
            let peeked_span = self.peek_token.into_span();
            self.advance_tokens();
            return Err(ParserError::UnexpectedTokenKind{span : peeked_span, expected : token_kind,found : peeked_kind })
        }
        self.advance_tokens();
        return Ok(());
    }

    // A program can contain many modules which themselves are programs
    pub fn parse_program(&mut self) -> Result<Program, &Vec<ParserError>> {
        let program = self.parse_unit(Token::EOF);
        if self.errors.len() > 0 {
            return Err(&self.errors)
        } else {
            return Ok(program)
        }
    }
    fn parse_unit(&mut self, delimeter : Token) -> Program {
        use super::prefix_parser::{FuncParser, UseParser, ModuleParser};

        let mut program = Program::with_capacity(self.lexer.by_ref().approx_len());

        while self.curr_token != delimeter {
            // First check if we have a function definition.
            // Function definitions are not added to the AST
            // Although we can have function literals starting with the function keyword
            // they will be self-contained within another function and they will start with a `let` token
            // Eg let add = fn(x,y) {x+y}
            match self.curr_token.clone().into() {
                Token::Attribute(attr) => {
                    self.advance_tokens(); // Skip the attribute
                    let func_def = FuncParser::parse_fn_definition(self, Some(attr));
                    self.on_value(func_def, |value|program.push_function(value));
                },
                Token::Keyword(Keyword::Fn) => {
                    let func_def = FuncParser::parse_fn_definition(self, None);
                    self.on_value(func_def, |value|program.push_function(value));
                }
                Token::Keyword(Keyword::Mod) => {
                    let parsed_mod = ModuleParser::parse_module_definition(self);
                    self.on_value(parsed_mod, |(module_identifier, module)|program.push_module(module_identifier, module));
                }
                Token::Keyword(Keyword::Use) => {
                    let import_stmt = UseParser::parse(self);
                    self.on_value(import_stmt, |value|program.push_import(value));
                }
                Token::Comment(_) => {
                    // This is a comment outside of a function.
                    // Currently we do nothing with Comment tokens
                    // It may be possible to store them in the AST, but this may not be helpful
                    // XXX: Maybe we can follow Rust and say by default all public functions need documentation?
                }
                _ => {
                    // Parse regular statements
                    let statement = self.parse_statement();
                    self.on_value(statement, |value|program.push_statement(value));
                }
            }
            // The current token will be the ending token for whichever branch was just picked
            // so we advance from that
            self.advance_tokens();
        }

        program
    }

    fn on_value<T, F>(&mut self, parser_res : ParserResult<T>, mut func : F) 
            where F: FnMut(T) 
    {
        match parser_res {
            Ok(value) => func(value),
            Err(err) => {
                self.errors.push(err);
                self.synchronise();
            }
        }
    }

    // For now the synchonisation strategy is basic
    fn synchronise(&mut self) {

        

        loop {
            
            if self.peek_token ==  Token::EOF
            {
                break
            } 

            if self.choose_prefix_parser().is_some() {
                self.advance_tokens();
                break
            }

            if self.peek_token ==  Token::Keyword(Keyword::Private) || 
            self.peek_token ==  Token::Keyword(Keyword::Let) || 
            self.peek_token ==  Token::Keyword(Keyword::Fn) {
                self.advance_tokens();
                break
            }
            if self.peek_token ==  Token::RightBrace ||
            self.peek_token ==  Token::Semicolon
            
            {
                self.advance_tokens();
                self.advance_tokens();
                break
            } 
            
            self.advance_tokens()    
        }
    }

    pub fn parse_module(&mut self) -> Program{
        self.parse_unit(Token::RightBrace)
    }
    pub fn parse_statement(&mut self) -> ParserStmtResult {
        use crate::parser::prefix_parser::{DeclarationParser, IfParser, ConstrainParser};

        // The first type of statement we could have is a variable declaration statement
        if self.curr_token.can_start_declaration() {
            return DeclarationParser::parse_declaration_statement(self);
        };

        let stmt = match self.curr_token.token() {
            tk if tk.is_comment() => {
                // Comments here are within a function
                self.advance_tokens();
                return self.parse_statement()
            }
            Token::Keyword(Keyword::Constrain) => {
                Statement::Constrain(ConstrainParser::parse_constrain_statement(self)?)
            }
            Token::Keyword(Keyword::If) => {
                Statement::If(IfParser::parse_if_statement(self)?)
            }
            _ => {
                let expr = self.parse_expression_statement()?;
                Statement::Expression(expr)
            }
        };
        // Check if the next token is a semi-colon(optional)
        if self.peek_token == Token::Semicolon {
            self.advance_tokens();
        };
        return Ok(stmt);
    }

    fn parse_expression_statement(&mut self) -> ParserExprResult {
        self.parse_expression(Precedence::Lowest)
    }

    pub(crate) fn parse_expression(&mut self, precedence: Precedence) -> ParserExprResult {

        // Calling this method means that we are at the beginning of a local expression
        // We may be in the middle of a global expression, but this does not matter
        let mut left_exp = match self.choose_prefix_parser() {
            Some(prefix_parser) => prefix_parser.parse(self)?,
            None => {
                return Err(ParserError::NoPrefixFunction{span : self.curr_token.into_span(), lexeme: self.curr_token.token().to_string()})
            }
        };

        while (self.peek_token != Token::Semicolon)
            && (precedence < Precedence::from(self.peek_token.token()))
        {
            match self.choose_infix_parser() {
                None => {
                    dbg!("No infix function found for {}", self.curr_token.token());
                    return Ok(left_exp.clone());
                }
                Some(infix_parser) => {
                    self.advance_tokens();
                    left_exp = infix_parser.parse(self, left_exp)?;
                }
            }
        }
        
        return Ok(left_exp);
    }
    fn choose_prefix_parser(&self) -> Option<PrefixParser> {
  
        match self.curr_token.token() {
            Token::Keyword(Keyword::For) => Some(PrefixParser::For),
            Token::LeftBracket => Some(PrefixParser::Array),
            x if x.kind() == TokenKind::Ident => Some(PrefixParser::Name),
            x if x.kind() == TokenKind::Literal => Some(PrefixParser::Literal),
            Token::Bang | Token::Minus => Some(PrefixParser::Unary),
            Token::LeftParen => Some(PrefixParser::Group),
            _ => None,
        }
    }
    fn choose_infix_parser(&self) -> Option<InfixParser> {
        match self.peek_token.token() {
            Token::Plus
            | Token::Minus
            | Token::Slash
            | Token::Pipe
            | Token::Ampersand
            | Token::Caret
            | Token::Star
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Equal
            | Token::Assign
            | Token::NotEqual => Some(InfixParser::Binary),
            Token::Keyword(Keyword::As) => Some(InfixParser::Cast),
            Token::LeftParen => Some(InfixParser::Call),
            Token::LeftBracket => Some(InfixParser::Index),
            Token::DoubleColon => Some(InfixParser::Path),
            _ => None,
        }
    }

    pub(crate) fn parse_block_statement(&mut self) -> Result<BlockStatement, ParserError> {
        let mut statements: Vec<Statement> = Vec::new();
        
        // Advance past the current token which is the left brace which was used to start the block statement
        // XXX: Check consistency with for parser, if parser and func parser
        self.advance_tokens();

        while (self.curr_token != Token::RightBrace) && (self.curr_token != Token::EOF) {
            statements.push(self.parse_statement()?);
            self.advance_tokens();
        }

        if self.curr_token != Token::RightBrace {
            return Err(ParserError::UnstructuredError{message : format!("Expected a }} to end the block statement"), span : self.curr_token.into_span()});
        }

        Ok(BlockStatement(statements))
    }

    pub(crate) fn parse_comma_separated_argument_list(
        &mut self,
        delimeter: Token,
    ) -> Result<Vec<Expression>, ParserError> {
        if self.peek_token == delimeter {
            self.advance_tokens();
            return Ok(Vec::new());
        }
        let mut arguments: Vec<Expression> = Vec::new();

        self.advance_tokens();
        arguments.push(self.parse_expression(Precedence::Lowest)?);
        while self.peek_token == Token::Comma {
            self.advance_tokens();
            self.advance_tokens();

            arguments.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.peek_check_variant_advance(&delimeter)?;

        Ok(arguments)
    }

    // Parse Types
    pub(crate) fn parse_type(&mut self) -> Result<Type, ParserError> {
        // Currently we only support the default types and integers.
        // If we get into this function, then the user is specifying a type
        match self.curr_token.token() {
            Token::Keyword(Keyword::Witness) => Ok(Type::Witness),
            Token::Keyword(Keyword::Public) => Ok(Type::Public),
            Token::Keyword(Keyword::Constant) => Ok(Type::Constant),
            Token::Keyword(Keyword::Field) => Ok(Type::FieldElement),
            Token::IntType(int_type) => Ok(int_type.into()),
            Token::LeftBracket => self.parse_array_type(),
            k => {
                let message = format!("Expected a type, found {}", k);
                return Err(ParserError::UnstructuredError{message, span : self.curr_token.into_span()});
            },
        }
    }
    
    fn parse_array_type(&mut self) -> Result<Type, ParserError> {
        // Expression is of the form [3]Type
    
        // Current token is '['
        //
        // Next token should be an Integer or right brace
        let array_len = match self.peek_token.clone().into() {
            Token::Int(integer) => {
                
                if !integer.fits_in_u128() {
                    let message = format!("Array sizes must fit within a u128");
                    return Err(ParserError::UnstructuredError{message, span: self.peek_token.into_span()});

                }
                self.advance_tokens();
                ArraySize::Fixed(integer.to_u128())
            },
            Token::RightBracket => ArraySize::Variable,
            _ => {
                let message = format!("The array size is defined as [k] for fixed size or [] for variable length. k must be a literal");
                return Err(ParserError::UnstructuredError{message, span: self.peek_token.into_span()});
            },
        };

        self.peek_check_variant_advance(&Token::RightBracket)?;
    
        // Skip Right bracket
        self.advance_tokens();
    
        // Disallow [4][3]Witness ie Matrices
        if self.peek_token == Token::LeftBracket {
           return Err(ParserError::UnstructuredError{message  : format!("Currently Multi-dimensional arrays are not supported"), span : self.peek_token.into_span()})
        }
    
        let array_type = self.parse_type()?;
    
        Ok(Type::Array(array_len, Box::new(array_type)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use noirc_errors::Spanned;
    use crate::ast::{
        BlockStatement, CallExpression, Expression, FunctionDefinition,
         Ident, IfStatement, InfixExpression, Literal, PrefixExpression,
        Statement, Type, BinaryOpKind
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

        let mut parser = Parser::new(Lexer::new(0,input));

        let program = parser.parse_program().unwrap();
        for (stmt, iden) in program.statements.iter().zip(test_iden.iter()) {
            helper_test_let(stmt, iden);
        }

        assert_eq!(program.statements.len(), 3);
    }

    fn helper_test_let(statement: &Statement, iden: &str) {
        // First make sure that the statement is a let statement
        let let_stmt = match statement {
            Statement::Let(stmt) => stmt,
            _ => unreachable!("Expected a let statement"),
        };

        // Now assert the correct identifier is in the let statement
        assert_eq!(let_stmt.identifier.0.contents, iden);
    }

    #[test]
    fn test_parse_identifier() {
        let input = "hello;world;This_is_a_word";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let test_iden = vec!["hello", "world", "This_is_a_word"];

        for (stmt, iden) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x,
                _ => unreachable!(),
            };
            // Extract the identifier
            let name = match expression.kind {
                ExpressionKind::Ident(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(iden, &name)
        }
    }

    #[test]
    fn test_parse_literals() {
        let input = "10;true;\"string_literal\"";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let test_iden = vec![
            Literal::Integer(10.into()),
            Literal::Bool(true),
            Literal::Str("string_literal".to_string()),
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x,
                _ => unreachable!(),
            };
            // Extract the literal
            let literal = match expression.kind {
                ExpressionKind::Literal(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(expected_lit, &literal)
        }
    }
    #[test]
    fn test_parse_prefix() {
        use crate::ast::*;
        let input = "!99;-100;!true";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let test_iden = vec![
            PrefixExpression {
                operator: UnaryOp::Not,
                rhs: ExpressionKind::Literal(Literal::Integer(99.into())).into_span(Default::default()),
            },
            PrefixExpression {
                operator: UnaryOp::Minus,
                rhs: ExpressionKind::Literal(Literal::Integer(100.into())).into_span(Default::default()),
            },
            PrefixExpression {
                operator: UnaryOp::Not,
                rhs: ExpressionKind::Literal(Literal::Bool(true)).into_span(Default::default()),
            },
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let spanned_expression = match stmt {
                Statement::Expression(x) => x,
                _ => unreachable!(),
            };
            // Extract the prefix expression
            let literal = match spanned_expression.kind {
                ExpressionKind::Prefix(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *literal)
        }
    }

    #[test]
    fn test_parse_infix() {
        let input = "5+5;10*5;true == false; false != false";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let test_iden = vec![
            InfixExpression {
                lhs: ExpressionKind::Literal(Literal::Integer(5.into())).into_span(Default::default()),
                operator: Spanned::from(Default::default(), BinaryOpKind::Add),
                rhs: ExpressionKind::Literal(Literal::Integer(5.into())).into_span(Default::default()),
            },
            InfixExpression {
                lhs: ExpressionKind::Literal(Literal::Integer(10.into())).into_span(Default::default()),
                operator: Spanned::from(Default::default(), BinaryOpKind::Multiply),
                rhs: ExpressionKind::Literal(Literal::Integer(5.into())).into_span(Default::default()),
            },
            InfixExpression {
                lhs: ExpressionKind::Literal(Literal::Bool(true)).into_span(Default::default()),
                operator: Spanned::from(Default::default(), BinaryOpKind::Equal),
                rhs: ExpressionKind::Literal(Literal::Bool(false)).into_span(Default::default()),
            },
            InfixExpression {
                lhs: ExpressionKind::Literal(Literal::Bool(false)).into_span(Default::default()),
                operator: Spanned::from(Default::default(), BinaryOpKind::NotEqual),
                rhs: ExpressionKind::Literal(Literal::Bool(false)).into_span(Default::default()),
            },
        ];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x,
                _ => unreachable!(),
            };
            // Extract the infix expression
            let literal = match expression.kind {
                ExpressionKind::Predicate(x) => x,
                ExpressionKind::Infix(x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *literal)
        }
    }
    #[test]
    fn test_parse_grouped() {
        use crate::ast::UnaryOp;

        let input = "-(5+10);-5+10";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        // Test the first expression : -(5+10)
        let grouped_expression = PrefixExpression {
            operator: UnaryOp::Minus,
            rhs: ExpressionKind::Infix(Box::new(InfixExpression {
                lhs: ExpressionKind::Literal(Literal::Integer(5.into())).into_span(Default::default()),
                operator: Spanned::from(Default::default(), BinaryOpKind::Add),
                rhs: ExpressionKind::Literal(Literal::Integer(10.into())).into_span(Default::default()),
            })).into_span(Default::default()),
        };

        let stmt = program.statements[0].clone();
        let expected_lit = grouped_expression;
        // Cast to an expression
        let expression = match stmt {
            Statement::Expression(x) => x,
            _ => unreachable!(),
        };
        // Extract the prefix expression
        let prefix = match expression.kind {
            ExpressionKind::Prefix(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*prefix, expected_lit);

        // Test the second expression : -5+10
        let ungrouped_expression = InfixExpression {
            lhs: ExpressionKind::Prefix(Box::new(PrefixExpression {
                operator: UnaryOp::Minus,
                rhs: ExpressionKind::Literal(Literal::Integer(5.into())).into_span(Default::default()),
            })).into_span(Default::default()),
            operator: Spanned::from(Default::default(), BinaryOpKind::Add),
            rhs: ExpressionKind::Literal(Literal::Integer(10.into())).into_span(Default::default()),
        };

        let stmt = program.statements[1].clone();
        let expected_lit = ungrouped_expression;
        // Cast to an expression
        let expression = match stmt {
            Statement::Expression(x) => x,
            _ => unreachable!(),
        };
        // Extract the prefix expression
        let prefix = match expression.kind {
            ExpressionKind::Infix(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*prefix, expected_lit);
    }

    #[test]
    fn test_parse_if_expression() {
        let input = "if (x < y) { x };";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let x_ident : Ident = String::from("x").into();
        let y_ident : Ident = String::from("y").into();

        let expected_if = IfStatement {
            condition: ExpressionKind::Predicate(Box::new(InfixExpression {
                lhs: x_ident.clone().into(),
                operator: Spanned::from(Default::default(), BinaryOpKind::Less),
                rhs: y_ident.into(),
            })).into_span(Default::default()),
            consequence: BlockStatement(vec![Statement::Expression(
                x_ident.into()
            )]),
            alternative: None,
        };

        let stmt = program.statements[0].clone();
        let if_stmt = match stmt {
            Statement::If(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(*if_stmt, expected_if)
    }
    #[test]
    fn test_parse_if_else_expression() {
        let input = "if (foo < bar) { cat } else {dog};";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let foo_ident : Ident = String::from("foo").into();
        let bar_ident : Ident = String::from("bar").into();
        let cat_ident : Ident = String::from("cat").into();
        let dog_ident : Ident = String::from("dog").into();

        let expected_if = IfStatement {
            condition: ExpressionKind::Predicate(Box::new(InfixExpression {
                lhs: foo_ident.into(),
                operator: Spanned::from(Default::default(), BinaryOpKind::Less),
                rhs: bar_ident.into(),
            })).into_span(Default::default()),
            consequence: BlockStatement(vec![Statement::Expression(
                cat_ident.into(),
            )]),
            alternative: Some(BlockStatement(vec![Statement::Expression(
                dog_ident.into(),
            )])),
        };

        let stmt = program.statements[0].clone();
        let if_stmt = match stmt {
            Statement::If(x) => x,
            _ => unreachable!(),
        };

        assert_eq!(*if_stmt, expected_if)
    }

    #[test]
    fn test_parse_int_type() {
        use crate::parser::prefix_parser::DeclarationParser;
        use crate::ast::{PrivateStatement, Signedness};
        let input = "priv x : i102 = a";
        let mut parser = Parser::new(Lexer::new(0,input));
        let stmt = DeclarationParser::parse_declaration_statement(&mut parser).unwrap();

        let x_ident : Ident = String::from("x").into();
        let a_ident : Ident = String::from("a").into();

        let priv_stmt_expected = PrivateStatement {
            identifier: x_ident,
            r#type: Type::Integer(Signedness::Signed, 102).into(),
            expression: a_ident.into(),
        };
        match stmt {
            Statement::Private(priv_stmt) => {
                assert_eq!(priv_stmt, priv_stmt_expected);
            }
            _ => unreachable!("Expected a private statement"),
        }
    }

    #[test]
    // XXX: This just duplicates most of test_funct_literal. Refactor to avoid duplicate code
    fn test_parse_function_def_literal() {
        let input = "fn add(x : Public,y : Constant){x+y}";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let x_ident : Ident = String::from("x").into();
        let y_ident : Ident = String::from("y").into();

        let parameters = vec![
            (x_ident.clone(), Type::Public),
            (y_ident.clone(), Type::Constant),
        ];

        let infix_expression = InfixExpression {
            lhs: x_ident.into(),
            operator: Spanned::from(Default::default(), BinaryOpKind::Add),
            rhs: y_ident.into(),
        };

        let add_ident : Ident = String::from("add").into();

        let expected = vec![FunctionDefinition {
            name: add_ident,
            attribute : None,
            parameters: parameters,
            body: BlockStatement(vec![Statement::Expression(
                ExpressionKind::Infix(Box::new(infix_expression)).into_span(Default::default()),
            )]),
            return_type : Type::Unit,       
         }];

        for (expected_def, got_def) in expected.into_iter().zip(program.functions.into_iter()) {
            assert_eq!(expected_def, got_def);
        }
    }

    #[test]
    fn test_parse_call_expression() {
        let input = "add(1,2+3)";
        let mut parser = Parser::new(Lexer::new(0,input));
        let program = parser.parse_program().unwrap();

        let add_ident : Ident = String::from("add").into();

        let test_iden = vec![CallExpression {
            func_name: add_ident,
            arguments: vec![
                ExpressionKind::Literal(Literal::Integer(1.into())).into_span(Default::default()),
                ExpressionKind::Infix(Box::new(InfixExpression {
                    lhs: ExpressionKind::Literal(Literal::Integer(2.into())).into_span(Default::default()),
                    operator: Spanned::from(Default::default(), BinaryOpKind::Add),
                    rhs: ExpressionKind::Literal(Literal::Integer(3.into())).into_span(Default::default()),
                })).into_span(Default::default()),
            ],
        }];

        for (stmt, expected_lit) in program.statements.into_iter().zip(test_iden.iter()) {
            // Cast to an expression
            let expression = match stmt {
                Statement::Expression(x) => x,
                _ => unreachable!(),
            };
            // Extract the function literal expression
            let call_expr = match expression.kind {
                ExpressionKind::Call(_,x) => x,
                _ => unreachable!(),
            };

            assert_eq!(*expected_lit, *call_expr)
        }
    }
}
