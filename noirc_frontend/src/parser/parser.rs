use super::{errors::ParserErrorKind, ParsedModule, Precedence};
use crate::ast::{ArraySize, Expression, ExpressionKind, Statement, Type};
use crate::lexer::Lexer;
use crate::token::{Keyword, SpannedToken, Token, TokenKind};

use super::infix_parser::InfixParser;
use super::prefix_parser::PrefixParser;

pub type ParserResult<T> = Result<T, ParserErrorKind>;
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
    pub(crate) errors: Vec<ParserErrorKind>,
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
    pub fn from_src(src: &'a str) -> Self {
        Parser::new(Lexer::new(src))
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
                }
                Err(lex_err) => self.errors.push(ParserErrorKind::LexerError(lex_err)),
            }
        }

        // At this point, we could check for lexer errors
        // and abort, however, if we do not we may be able to
        // recover if the next token is the correct one
        //
        // Its also usually bad UX to only show one error at a time.
    }

    // peaks at the next token
    // asserts that it should be of a certain variant
    // If it is, the parser is advanced
    pub(crate) fn peek_check_variant_advance(
        &mut self,
        token: &Token,
    ) -> Result<(), ParserErrorKind> {
        let same_variant = self.peek_token.is_variant(token);

        if !same_variant {
            let peeked_span = self.peek_token.into_span();
            let peeked_token = self.peek_token.token().clone();
            self.advance_tokens(); // We advance the token regardless, so the parser does not choke on a prefix function
            return Err(ParserErrorKind::UnexpectedToken {
                span: peeked_span,
                expected: token.clone(),
                found: peeked_token,
            });
        }
        self.advance_tokens();
        return Ok(());
    }

    // peaks at the next token
    // asserts that it should be of a certain kind
    // If it is, the parser is advanced
    pub(crate) fn peek_check_kind_advance(
        &mut self,
        token_kind: TokenKind,
    ) -> Result<(), ParserErrorKind> {
        let peeked_kind = self.peek_token.kind();
        let same_kind = peeked_kind == token_kind;
        if !same_kind {
            let peeked_span = self.peek_token.into_span();
            self.advance_tokens();
            return Err(ParserErrorKind::UnexpectedTokenKind {
                span: peeked_span,
                expected: token_kind,
                found: peeked_kind,
            });
        }
        self.advance_tokens();
        return Ok(());
    }

    /// A Program corresponds to a single module
    pub fn parse_program(&mut self) -> Result<ParsedModule, &Vec<ParserErrorKind>> {
        use super::prefix_parser::{FuncParser, ModuleParser, UseParser};

        let mut program = ParsedModule::with_capacity(self.lexer.by_ref().approx_len());

        while self.curr_token != Token::EOF {
            match self.curr_token.clone().into() {
                Token::Attribute(attr) => {
                    self.advance_tokens(); // Skip the attribute
                    let func_def = FuncParser::parse_fn_definition(self, Some(attr));
                    self.on_value(func_def, |value| program.push_function(value));
                }
                Token::Keyword(Keyword::Fn) => {
                    let func_def = FuncParser::parse_fn_definition(self, None);
                    self.on_value(func_def, |value| program.push_function(value));
                }
                Token::Keyword(Keyword::Mod) => {
                    let parsed_mod = ModuleParser::parse_decl(self);
                    self.on_value(parsed_mod, |module_identifier| {
                        program.push_module_decl(module_identifier)
                    });
                }
                Token::Keyword(Keyword::Use) => {
                    let import_stmt = UseParser::parse(self);
                    self.on_value(import_stmt, |value| program.push_import(value));
                }
                Token::Comment(_) => {
                    // This is a comment outside of a function.
                    // Currently we do nothing with Comment tokens
                    // It may be possible to store them in the AST, but this may not be helpful
                    // XXX: Maybe we can follow Rust and say by default all public functions need documentation?
                }
                tok => {
                    // XXX: We may allow global constants. We can use a subenum to remove the wildcard pattern
                    let expected_tokens = r#" expected "`mod`, `use`,`fn` `#`"#;
                    let err = ParserErrorKind::UnstructuredError {
                        span: self.curr_token.into_span(),
                        message: format!("found `{}`. {}", tok, expected_tokens), // XXX: Fix in next refactor, avoid allocations with error messages
                    };
                    self.errors.push(err);
                    return Err(&self.errors);
                }
            }
            // The current token will be the ending token for whichever branch was just picked
            // so we advance from that
            self.advance_tokens();
        }

        if self.errors.len() > 0 {
            return Err(&self.errors);
        } else {
            return Ok(program);
        }
    }

    fn on_value<T, F>(&mut self, parser_res: ParserResult<T>, mut func: F)
    where
        F: FnMut(T),
    {
        match parser_res {
            Ok(value) => func(value),
            Err(err) => {
                self.errors.push(err);
                self.synchronise();
            }
        }
    }

    // XXX: For now the synchonisation strategy is basic.
    // XXX: Revise this after error refactoring is completed
    fn synchronise(&mut self) {
        loop {
            if self.peek_token == Token::EOF {
                break;
            }

            if self.choose_prefix_parser().is_some() {
                self.advance_tokens();
                break;
            }

            if self.peek_token == Token::Keyword(Keyword::Private)
                || self.peek_token == Token::Keyword(Keyword::Let)
                || self.peek_token == Token::Keyword(Keyword::Fn)
            {
                self.advance_tokens();
                break;
            }
            if self.peek_token == Token::RightBrace || self.peek_token == Token::Semicolon {
                self.advance_tokens();
                self.advance_tokens();
                break;
            }

            self.advance_tokens()
        }
    }

    pub fn parse_statement(&mut self) -> ParserStmtResult {
        use crate::parser::prefix_parser::{ConstrainParser, DeclarationParser};

        let stmt = match self.curr_token.token() {
            tk if tk.can_start_declaration() => {
                return DeclarationParser::parse_statement(self);
            }
            tk if tk.is_comment() => {
                // Comments here are within a function
                self.advance_tokens();
                return self.parse_statement();
            }
            Token::Keyword(Keyword::Constrain) => {
                Statement::Constrain(ConstrainParser::parse_statement(self)?)
            }
            _ => {
                let expr = self.parse_expression_statement()?;

                // Check if the next token is a semi-colon
                // If it is, it is a SemiExpr
                if self.peek_token == Token::Semicolon {
                    self.advance_tokens();
                    Statement::Semi(expr)
                } else {
                    Statement::Expression(expr)
                }
            }
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
                return Err(ParserErrorKind::ExpectedExpression {
                    span: self.curr_token.into_span(),
                    lexeme: self.curr_token.token().to_string(),
                })
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
            Token::Keyword(Keyword::If) => Some(PrefixParser::If),
            Token::Keyword(Keyword::For) => Some(PrefixParser::For),
            Token::LeftBracket => Some(PrefixParser::Array),
            x if x.kind() == TokenKind::Ident => Some(PrefixParser::Path),
            x if x.kind() == TokenKind::Literal => Some(PrefixParser::Literal),
            Token::Bang | Token::Minus => Some(PrefixParser::Unary),
            Token::LeftParen => Some(PrefixParser::Group),
            Token::LeftBrace => Some(PrefixParser::Block),
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
            _ => None,
        }
    }

    /// Parse a comma separated list with a chosen delimiter
    ///
    /// This function is used to parse arrays and call expressions.
    /// It is very similar to `parse_fn_parameters`, in the future
    /// these methods may be unified.
    ///
    /// Cursor Start : `START_TOKEN`
    ///
    /// Cursor End : `END_TOKEN`
    ///
    /// Importantly note that the cursor should be on the starting token
    /// and not the first element in the list.
    ///
    /// Example : [a,b,c]
    /// START_TOKEN = `[`
    /// END_TOKEN = `]`
    ///
    /// In general, the END_TOKEN will be closing_token
    pub(crate) fn parse_comma_separated_argument_list(
        &mut self,
        closing_token: Token,
    ) -> Result<Vec<Expression>, ParserErrorKind> {
        // An empty container.
        // Advance to the ending token and
        // return an empty array
        if self.peek_token == closing_token {
            self.advance_tokens();
            return Ok(Vec::new());
        }
        let mut arguments: Vec<Expression> = Vec::new();

        // Parse the first element, implicitly assuming that `parse_expression`
        // does not advance the token from what it has just parsed
        self.advance_tokens();
        arguments.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token == Token::Comma {
            self.advance_tokens();

            if (self.curr_token == Token::Comma) && (self.peek_token == closing_token) {
                // Entering here means there is nothing else to parse;
                // the list has a trailing comma
                break;
            }

            self.advance_tokens();

            arguments.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.peek_check_variant_advance(&closing_token)?;

        Ok(arguments)
    }

    /// Cursor Start : `TYPE`
    ///
    /// Cursor End : `TYPE`
    /// The cursor starts on the first token which represents the type
    /// It ends on the last token in the Type
    pub(crate) fn parse_type(&mut self) -> Result<Type, ParserErrorKind> {
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
                return Err(ParserErrorKind::UnstructuredError {
                    message,
                    span: self.curr_token.into_span(),
                });
            }
        }
    }

    fn parse_array_type(&mut self) -> Result<Type, ParserErrorKind> {
        // Expression is of the form [3]Type

        // Current token is '['
        //
        // Next token should be an Integer or right brace
        let array_len = match self.peek_token.clone().into() {
            Token::Int(integer) => {
                if !integer.fits_in_u128() {
                    let message = format!("Array sizes must fit within a u128");
                    return Err(ParserErrorKind::UnstructuredError {
                        message,
                        span: self.peek_token.into_span(),
                    });
                }
                self.advance_tokens();
                ArraySize::Fixed(integer.to_u128())
            }
            Token::RightBracket => ArraySize::Variable,
            _ => {
                let message = format!("The array size is defined as [k] for fixed size or [] for variable length. k must be a literal");
                return Err(ParserErrorKind::UnstructuredError {
                    message,
                    span: self.peek_token.into_span(),
                });
            }
        };

        self.peek_check_variant_advance(&Token::RightBracket)?;

        // Skip Right bracket
        self.advance_tokens();

        // Disallow [4][3]Witness ie Matrices
        if self.peek_token == Token::LeftBracket {
            return Err(ParserErrorKind::UnstructuredError {
                message: format!("Currently Multi-dimensional arrays are not supported"),
                span: self.peek_token.into_span(),
            });
        }

        let array_type = self.parse_type()?;

        Ok(Type::Array(array_len, Box::new(array_type)))
    }
}
