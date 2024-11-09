use std::sync::Arc;

use super::{
    ir::{instruction::BinaryOp, types::Type},
    Ssa,
};

use acvm::{AcirField, FieldElement};
use ast::{
    Identifier, ParsedBlock, ParsedFunction, ParsedInstruction, ParsedParameter, ParsedSsa,
    ParsedValue,
};
use lexer::{Lexer, LexerError};
use noirc_errors::Span;
use noirc_frontend::{monomorphization::ast::InlineType, token::IntType};
use token::{Keyword, SpannedToken, Token};

use crate::ssa::{ir::function::RuntimeType, parser::ast::ParsedTerminator};

mod ast;
mod into_ssa;
mod lexer;
mod tests;
mod token;

impl Ssa {
    pub(crate) fn from_str(src: &str) -> Result<Ssa, SsaError> {
        let mut parser = Parser::new(src).map_err(SsaError::ParserError)?;
        let parsed_ssa = parser.parse_ssa().map_err(SsaError::ParserError)?;
        parsed_ssa.into_ssa()
    }
}

#[derive(Debug)]
pub(crate) enum SsaError {
    ParserError(ParserError),
    UnknownVariable(Identifier),
    UnknownBlock(Identifier),
    UnknownFunction(Identifier),
    MismatchedReturnValues { returns: Vec<Identifier>, expected: usize },
    VariableAlreadyDefined(Identifier),
}

type ParseResult<T> = Result<T, ParserError>;

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    token: SpannedToken,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> ParseResult<Self> {
        let lexer = Lexer::new(source);
        let mut parser = Self { lexer, token: eof_spanned_token() };
        parser.token = parser.read_token_internal()?;
        Ok(parser)
    }

    pub(crate) fn parse_ssa(&mut self) -> ParseResult<ParsedSsa> {
        let mut functions = Vec::new();
        while !self.at(Token::Eof) {
            let function = self.parse_function()?;
            functions.push(function);
        }
        Ok(ParsedSsa { functions })
    }

    fn parse_function(&mut self) -> ParseResult<ParsedFunction> {
        let runtime_type = self.parse_runtime_type()?;
        self.eat_or_error(Token::Keyword(Keyword::Fn))?;

        let external_name = self.eat_ident_or_error()?;
        let internal_name = self.eat_ident_or_error()?;

        self.eat_or_error(Token::LeftBrace)?;

        let blocks = self.parse_blocks()?;

        self.eat_or_error(Token::RightBrace)?;

        Ok(ParsedFunction { runtime_type, external_name, internal_name, blocks })
    }

    fn parse_runtime_type(&mut self) -> ParseResult<RuntimeType> {
        let acir = if self.eat_keyword(Keyword::Acir)? {
            true
        } else if self.eat_keyword(Keyword::Brillig)? {
            false
        } else {
            return self.expected_one_of_tokens(&[
                Token::Keyword(Keyword::Acir),
                Token::Keyword(Keyword::Brillig),
            ]);
        };

        self.eat_or_error(Token::LeftParen)?;
        let inline_type = self.parse_inline_type()?;
        self.eat_or_error(Token::RightParen)?;

        if acir {
            Ok(RuntimeType::Acir(inline_type))
        } else {
            Ok(RuntimeType::Brillig(inline_type))
        }
    }

    fn parse_inline_type(&mut self) -> ParseResult<InlineType> {
        if self.eat_keyword(Keyword::Inline)? {
            Ok(InlineType::Inline)
        } else if self.eat_keyword(Keyword::InlineAlways)? {
            Ok(InlineType::InlineAlways)
        } else if self.eat_keyword(Keyword::Fold)? {
            Ok(InlineType::Fold)
        } else if self.eat_keyword(Keyword::NoPredicates)? {
            Ok(InlineType::NoPredicates)
        } else {
            self.expected_one_of_tokens(&[
                Token::Keyword(Keyword::Inline),
                Token::Keyword(Keyword::InlineAlways),
                Token::Keyword(Keyword::Fold),
                Token::Keyword(Keyword::NoPredicates),
            ])
        }
    }

    fn parse_blocks(&mut self) -> ParseResult<Vec<ParsedBlock>> {
        let mut blocks = Vec::new();
        while !self.at(Token::RightBrace) {
            let block = self.parse_block()?;
            blocks.push(block);
        }
        Ok(blocks)
    }

    fn parse_block(&mut self) -> ParseResult<ParsedBlock> {
        let name = self.eat_ident_or_error()?;
        self.eat_or_error(Token::LeftParen)?;

        let mut parameters = Vec::new();
        while !self.at(Token::RightParen) {
            parameters.push(self.parse_parameter()?);
            if !self.eat(Token::Comma)? {
                break;
            }
        }

        self.eat_or_error(Token::RightParen)?;
        self.eat_or_error(Token::Colon)?;

        let instructions = self.parse_instructions()?;
        let terminator = self.parse_terminator()?;
        Ok(ParsedBlock { name, parameters, instructions, terminator })
    }

    fn parse_parameter(&mut self) -> ParseResult<ParsedParameter> {
        let identifier = self.eat_identifier_or_error()?;
        self.eat_or_error(Token::Colon)?;
        let typ = self.parse_type()?;
        Ok(ParsedParameter { identifier, typ })
    }

    fn parse_instructions(&mut self) -> ParseResult<Vec<ParsedInstruction>> {
        let mut instructions = Vec::new();
        while let Some(instruction) = self.parse_instruction()? {
            instructions.push(instruction);
        }
        Ok(instructions)
    }

    fn parse_instruction(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if let Some(instruction) = self.parse_call()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_constrain()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_decrement_rc()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_enable_side_effects()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_increment_rc()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_range_check()? {
            return Ok(Some(instruction));
        }

        if let Some(instruction) = self.parse_store()? {
            return Ok(Some(instruction));
        }

        if let Some(target) = self.eat_identifier()? {
            let mut targets = vec![target];

            while self.eat(Token::Comma)? {
                let target = self.eat_identifier_or_error()?;
                targets.push(target);
            }

            self.eat_or_error(Token::Assign)?;

            if self.eat_keyword(Keyword::Call)? {
                let function = self.eat_identifier_or_error()?;
                let arguments = self.parse_arguments()?;
                self.eat_or_error(Token::Arrow)?;
                let types = self.parse_types()?;
                return Ok(Some(ParsedInstruction::Call { targets, function, arguments, types }));
            }

            if targets.len() > 1 {
                return Err(ParserError::MultipleReturnValuesOnlyAllowedForCall {
                    second_target: targets[1].clone(),
                });
            }

            let target = targets.remove(0);

            if self.eat_keyword(Keyword::Allocate)? {
                self.eat_or_error(Token::Arrow)?;
                let typ = self.parse_type()?;
                return Ok(Some(ParsedInstruction::Allocate { target, typ }));
            }

            if self.eat_keyword(Keyword::ArrayGet)? {
                let array = self.parse_value_or_error()?;
                self.eat_or_error(Token::Comma)?;
                self.eat_or_error(Token::Keyword(Keyword::Index))?;
                let index = self.parse_value_or_error()?;
                self.eat_or_error(Token::Arrow)?;
                let element_type = self.parse_type()?;
                return Ok(Some(ParsedInstruction::ArrayGet {
                    target,
                    element_type,
                    array,
                    index,
                }));
            }

            if self.eat_keyword(Keyword::ArraySet)? {
                let array = self.parse_value_or_error()?;
                self.eat_or_error(Token::Comma)?;
                self.eat_or_error(Token::Keyword(Keyword::Index))?;
                let index = self.parse_value_or_error()?;
                self.eat_or_error(Token::Comma)?;
                self.eat_or_error(Token::Keyword(Keyword::Value))?;
                let value = self.parse_value_or_error()?;
                return Ok(Some(ParsedInstruction::ArraySet { target, array, index, value }));
            }

            if self.eat_keyword(Keyword::Cast)? {
                let lhs = self.parse_value_or_error()?;
                self.eat_or_error(Token::Keyword(Keyword::As))?;
                let typ = self.parse_type()?;
                return Ok(Some(ParsedInstruction::Cast { target, lhs, typ }));
            }

            if self.eat_keyword(Keyword::Load)? {
                let value = self.parse_value_or_error()?;
                self.eat_or_error(Token::Arrow)?;
                let typ = self.parse_type()?;
                return Ok(Some(ParsedInstruction::Load { target, value, typ }));
            }

            if self.eat_keyword(Keyword::Not)? {
                let value = self.parse_value_or_error()?;
                return Ok(Some(ParsedInstruction::Not { target, value }));
            }

            if self.eat_keyword(Keyword::Truncate)? {
                let value = self.parse_value_or_error()?;
                self.eat_or_error(Token::Keyword(Keyword::To))?;
                let bit_size = self.eat_int_or_error()?.to_u128() as u32;
                self.eat_or_error(Token::Keyword(Keyword::Bits))?;
                self.eat_or_error(Token::Comma)?;
                self.eat_or_error(Token::Keyword(Keyword::MaxBitSize))?;
                self.eat_or_error(Token::Colon)?;
                let max_bit_size = self.eat_int_or_error()?.to_u128() as u32;
                return Ok(Some(ParsedInstruction::Truncate {
                    target,
                    value,
                    bit_size,
                    max_bit_size,
                }));
            }

            if let Some(op) = self.eat_binary_op()? {
                let lhs = self.parse_value_or_error()?;
                self.eat_or_error(Token::Comma)?;
                let rhs = self.parse_value_or_error()?;
                return Ok(Some(ParsedInstruction::BinaryOp { target, lhs, op, rhs }));
            }

            return self.expected_instruction_or_terminator();
        }

        Ok(None)
    }

    fn eat_binary_op(&mut self) -> ParseResult<Option<BinaryOp>> {
        if self.eat_keyword(Keyword::Add)? {
            return Ok(Some(BinaryOp::Add));
        }

        if self.eat_keyword(Keyword::Sub)? {
            return Ok(Some(BinaryOp::Sub));
        }

        if self.eat_keyword(Keyword::Mul)? {
            return Ok(Some(BinaryOp::Mul));
        }

        if self.eat_keyword(Keyword::Div)? {
            return Ok(Some(BinaryOp::Div));
        }

        if self.eat_keyword(Keyword::Eq)? {
            return Ok(Some(BinaryOp::Eq));
        }

        if self.eat_keyword(Keyword::Mod)? {
            return Ok(Some(BinaryOp::Mod));
        }

        if self.eat_keyword(Keyword::Lt)? {
            return Ok(Some(BinaryOp::Lt));
        }

        if self.eat_keyword(Keyword::And)? {
            return Ok(Some(BinaryOp::And));
        }

        if self.eat_keyword(Keyword::Or)? {
            return Ok(Some(BinaryOp::Or));
        }

        if self.eat_keyword(Keyword::Xor)? {
            return Ok(Some(BinaryOp::Xor));
        }

        if self.eat_keyword(Keyword::Shl)? {
            return Ok(Some(BinaryOp::Shl));
        }

        if self.eat_keyword(Keyword::Shr)? {
            return Ok(Some(BinaryOp::Shr));
        }

        Ok(None)
    }

    fn parse_call(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::Call)? {
            return Ok(None);
        }

        let function = self.eat_identifier_or_error()?;
        let arguments = self.parse_arguments()?;
        Ok(Some(ParsedInstruction::Call { targets: vec![], function, arguments, types: vec![] }))
    }

    fn parse_constrain(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::Constrain)? {
            return Ok(None);
        }

        let lhs = self.parse_value_or_error()?;
        self.eat_or_error(Token::Equal)?;
        let rhs = self.parse_value_or_error()?;
        Ok(Some(ParsedInstruction::Constrain { lhs, rhs }))
    }

    fn parse_decrement_rc(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::DecRc)? {
            return Ok(None);
        }

        let value = self.parse_value_or_error()?;
        Ok(Some(ParsedInstruction::DecrementRc { value }))
    }

    fn parse_enable_side_effects(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::EnableSideEffects)? {
            return Ok(None);
        }

        let condition = self.parse_value_or_error()?;
        Ok(Some(ParsedInstruction::EnableSideEffectsIf { condition }))
    }

    fn parse_increment_rc(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::IncRc)? {
            return Ok(None);
        }

        let value = self.parse_value_or_error()?;
        Ok(Some(ParsedInstruction::IncrementRc { value }))
    }

    fn parse_range_check(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::RangeCheck)? {
            return Ok(None);
        }

        let value = self.parse_value_or_error()?;
        self.eat_or_error(Token::Keyword(Keyword::To))?;
        let max_bit_size = self.eat_int_or_error()?.to_u128() as u32;
        self.eat_or_error(Token::Keyword(Keyword::Bits))?;
        Ok(Some(ParsedInstruction::RangeCheck { value, max_bit_size }))
    }

    fn parse_store(&mut self) -> ParseResult<Option<ParsedInstruction>> {
        if !self.eat_keyword(Keyword::Store)? {
            return Ok(None);
        }

        let value = self.parse_value_or_error()?;
        self.eat_or_error(Token::Keyword(Keyword::At))?;
        let address = self.parse_value_or_error()?;
        Ok(Some(ParsedInstruction::Store { address, value }))
    }

    fn parse_terminator(&mut self) -> ParseResult<ParsedTerminator> {
        if let Some(terminator) = self.parse_return()? {
            return Ok(terminator);
        }

        if let Some(terminator) = self.parse_jmp()? {
            return Ok(terminator);
        }

        if let Some(terminator) = self.parse_jmpif()? {
            return Ok(terminator);
        }

        self.expected_instruction_or_terminator()
    }

    fn parse_return(&mut self) -> ParseResult<Option<ParsedTerminator>> {
        // Before advancing to the next token (after a potential return keyword),
        // we check if a newline follows. This is because if we have this:
        //
        //   return
        // b1():
        //   ...
        //
        // then unless we check for a newline we can't know if the return
        // returns `b1` or not (we could check if a parentheses comes next, but
        // that would require a look-ahead and, for the purpose of the SSA parser,
        // it's just simpler to check if a newline follows)
        let newline_follows = self.newline_follows();

        if !self.eat_keyword(Keyword::Return)? {
            return Ok(None);
        }

        let values =
            if newline_follows { Vec::new() } else { self.parse_comma_separated_values()? };
        Ok(Some(ParsedTerminator::Return(values)))
    }

    fn parse_jmp(&mut self) -> ParseResult<Option<ParsedTerminator>> {
        if !self.eat_keyword(Keyword::Jmp)? {
            return Ok(None);
        }

        let destination = self.eat_identifier_or_error()?;
        let arguments = self.parse_arguments()?;
        Ok(Some(ParsedTerminator::Jmp { destination, arguments }))
    }

    fn parse_jmpif(&mut self) -> ParseResult<Option<ParsedTerminator>> {
        if !self.eat_keyword(Keyword::Jmpif)? {
            return Ok(None);
        }

        let condition = self.parse_value_or_error()?;
        self.eat_or_error(Token::Keyword(Keyword::Then))?;
        self.eat_or_error(Token::Colon)?;
        let then_block = self.eat_identifier_or_error()?;
        self.eat_or_error(Token::Comma)?;
        self.eat_or_error(Token::Keyword(Keyword::Else))?;
        self.eat_or_error(Token::Colon)?;
        let else_block = self.eat_identifier_or_error()?;

        Ok(Some(ParsedTerminator::Jmpif { condition, then_block, else_block }))
    }

    fn parse_arguments(&mut self) -> ParseResult<Vec<ParsedValue>> {
        self.eat_or_error(Token::LeftParen)?;
        let arguments = self.parse_comma_separated_values()?;
        self.eat_or_error(Token::RightParen)?;
        Ok(arguments)
    }

    fn parse_comma_separated_values(&mut self) -> ParseResult<Vec<ParsedValue>> {
        let mut values = Vec::new();
        while let Some(value) = self.parse_value()? {
            values.push(value);
            if !self.eat(Token::Comma)? {
                break;
            }
        }
        Ok(values)
    }

    fn parse_value_or_error(&mut self) -> ParseResult<ParsedValue> {
        if let Some(value) = self.parse_value()? {
            Ok(value)
        } else {
            self.expected_value()
        }
    }

    fn parse_value(&mut self) -> ParseResult<Option<ParsedValue>> {
        if let Some(value) = self.parse_field_value()? {
            return Ok(Some(value));
        }

        if let Some(value) = self.parse_int_value()? {
            return Ok(Some(value));
        }

        if let Some(value) = self.parse_array_value()? {
            return Ok(Some(value));
        }

        if let Some(identifier) = self.eat_identifier()? {
            return Ok(Some(ParsedValue::Variable(identifier)));
        }

        Ok(None)
    }

    fn parse_field_value(&mut self) -> ParseResult<Option<ParsedValue>> {
        if self.eat_keyword(Keyword::Field)? {
            let constant = self.eat_int_or_error()?;
            Ok(Some(ParsedValue::NumericConstant { constant, typ: Type::field() }))
        } else {
            Ok(None)
        }
    }

    fn parse_int_value(&mut self) -> ParseResult<Option<ParsedValue>> {
        if let Some(int_type) = self.eat_int_type()? {
            let constant = self.eat_int_or_error()?;
            let typ = match int_type {
                IntType::Unsigned(bit_size) => Type::unsigned(bit_size),
                IntType::Signed(bit_size) => Type::signed(bit_size),
            };
            Ok(Some(ParsedValue::NumericConstant { constant, typ }))
        } else {
            Ok(None)
        }
    }

    fn parse_array_value(&mut self) -> ParseResult<Option<ParsedValue>> {
        if self.eat(Token::LeftBracket)? {
            let values = self.parse_comma_separated_values()?;
            self.eat_or_error(Token::RightBracket)?;
            self.eat_or_error(Token::Keyword(Keyword::Of))?;
            let types = self.parse_types()?;
            let types_len = types.len();
            let values_len = values.len();
            Ok(Some(ParsedValue::Array {
                typ: Type::Array(Arc::new(types), values_len / types_len),
                values,
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_types(&mut self) -> ParseResult<Vec<Type>> {
        if self.eat(Token::LeftParen)? {
            let types = self.parse_comma_separated_types()?;
            self.eat_or_error(Token::RightParen)?;
            Ok(types)
        } else {
            Ok(vec![self.parse_type()?])
        }
    }

    fn parse_comma_separated_types(&mut self) -> ParseResult<Vec<Type>> {
        let mut types = Vec::new();
        loop {
            let typ = self.parse_type()?;
            types.push(typ);
            if !self.eat(Token::Comma)? {
                break;
            }
        }
        Ok(types)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        if self.eat_keyword(Keyword::Bool)? {
            return Ok(Type::bool());
        }

        if self.eat_keyword(Keyword::Field)? {
            return Ok(Type::field());
        }

        if let Some(int_type) = self.eat_int_type()? {
            return Ok(match int_type {
                IntType::Unsigned(bit_size) => Type::unsigned(bit_size),
                IntType::Signed(bit_size) => Type::signed(bit_size),
            });
        }

        if self.eat(Token::LeftBracket)? {
            let element_types = self.parse_types()?;
            self.eat_or_error(Token::Semicolon)?;
            let length = self.eat_int_or_error()?;
            self.eat_or_error(Token::RightBracket)?;
            return Ok(Type::Array(Arc::new(element_types), length.to_u128() as usize));
        }

        self.expected_type()
    }

    fn eat_identifier_or_error(&mut self) -> ParseResult<Identifier> {
        if let Some(identifier) = self.eat_identifier()? {
            Ok(identifier)
        } else {
            self.expected_identifier()
        }
    }

    fn eat_identifier(&mut self) -> ParseResult<Option<Identifier>> {
        let span = self.token.to_span();
        if let Some(name) = self.eat_ident()? {
            Ok(Some(Identifier::new(name, span)))
        } else {
            Ok(None)
        }
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> ParseResult<bool> {
        if let Token::Keyword(kw) = self.token.token() {
            if *kw == keyword {
                self.bump()?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn eat_ident(&mut self) -> ParseResult<Option<String>> {
        if !matches!(self.token.token(), Token::Ident(..)) {
            return Ok(None);
        }

        let token = self.bump()?;
        match token.into_token() {
            Token::Ident(ident) => Ok(Some(ident)),
            _ => unreachable!(),
        }
    }

    fn eat_ident_or_error(&mut self) -> ParseResult<String> {
        if let Some(ident) = self.eat_ident()? {
            Ok(ident)
        } else {
            self.expected_identifier()
        }
    }

    fn eat_int(&mut self) -> ParseResult<Option<FieldElement>> {
        if matches!(self.token.token(), Token::Int(..)) {
            let token = self.bump()?;
            match token.into_token() {
                Token::Int(int) => Ok(Some(int)),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn eat_int_or_error(&mut self) -> ParseResult<FieldElement> {
        if let Some(int) = self.eat_int()? {
            Ok(int)
        } else {
            self.expected_int()
        }
    }

    fn eat_int_type(&mut self) -> ParseResult<Option<IntType>> {
        let is_int_type = matches!(self.token.token(), Token::IntType(..));
        if is_int_type {
            let token = self.bump()?;
            match token.into_token() {
                Token::IntType(int_type) => Ok(Some(int_type)),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn eat(&mut self, token: Token) -> ParseResult<bool> {
        if self.token.token() == &token {
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn eat_or_error(&mut self, token: Token) -> ParseResult<()> {
        if self.eat(token.clone())? {
            Ok(())
        } else {
            self.expected_token(token)
        }
    }

    fn at(&self, token: Token) -> bool {
        self.token.token() == &token
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        self.at(Token::Keyword(keyword))
    }

    fn newline_follows(&self) -> bool {
        self.lexer.newline_follows()
    }

    fn bump(&mut self) -> ParseResult<SpannedToken> {
        let token = self.read_token_internal()?;
        Ok(std::mem::replace(&mut self.token, token))
    }

    fn read_token_internal(&mut self) -> ParseResult<SpannedToken> {
        self.lexer.next_token().map_err(ParserError::LexerError)
    }

    fn expected_instruction_or_terminator<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedInstructionOrTerminator {
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_identifier<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedIdentifier {
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_int<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedInt {
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_type<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedType {
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_value<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedValue {
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_token<T>(&mut self, token: Token) -> ParseResult<T> {
        Err(ParserError::ExpectedToken {
            token,
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }

    fn expected_one_of_tokens<T>(&mut self, tokens: &[Token]) -> ParseResult<T> {
        Err(ParserError::ExpectedOneOfTokens {
            tokens: tokens.to_vec(),
            found: self.token.token().clone(),
            span: self.token.to_span(),
        })
    }
}

#[derive(Debug)]
pub(crate) enum ParserError {
    LexerError(LexerError),
    ExpectedToken { token: Token, found: Token, span: Span },
    ExpectedOneOfTokens { tokens: Vec<Token>, found: Token, span: Span },
    ExpectedIdentifier { found: Token, span: Span },
    ExpectedInt { found: Token, span: Span },
    ExpectedType { found: Token, span: Span },
    ExpectedInstructionOrTerminator { found: Token, span: Span },
    ExpectedValue { found: Token, span: Span },
    MultipleReturnValuesOnlyAllowedForCall { second_target: Identifier },
}

fn eof_spanned_token() -> SpannedToken {
    SpannedToken::new(Token::Eof, Default::default())
}
