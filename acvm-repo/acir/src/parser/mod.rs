use std::{collections::BTreeSet, str::FromStr};

use acir_field::{AcirField, FieldElement};

use lexer::{Lexer, LexerError};
use noirc_errors::Span;
use thiserror::Error;
use token::{Keyword, SpannedToken, Token};

use crate::{
    BlackBoxFunc,
    circuit::{
        Circuit, Opcode, PublicInputs,
        opcodes::{BlackBoxFuncCall, ConstantOrWitnessEnum, FunctionInput},
    },
    native_types::{Expression, Witness},
};

mod lexer;
mod tests;
mod token;

pub struct AcirParserErrorWithSource {
    src: String,
    error: ParserError,
}

impl AcirParserErrorWithSource {
    fn parse_error(error: ParserError, src: &str) -> Self {
        Self { src: src.to_string(), error }
    }
}

impl std::fmt::Debug for AcirParserErrorWithSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.error.span();

        let mut byte: usize = 0;
        for line in self.src.lines() {
            let has_error =
                byte <= span.start() as usize && span.end() as usize <= byte + line.len();
            if has_error {
                writeln!(f)?;
            }

            writeln!(f, "{line}")?;

            if has_error {
                let offset = span.start() as usize - byte;
                write!(f, "{}", " ".repeat(offset))?;
                writeln!(f, "{}", "^".repeat((span.end() - span.start()) as usize))?;
                write!(f, "{}", " ".repeat(offset))?;
                writeln!(f, "{}", self.error)?;
                writeln!(f)?;
            }

            byte += line.len() + 1; // "+ 1" for the newline
        }
        Ok(())
    }
}

impl FromStr for Circuit<FieldElement> {
    type Err = AcirParserErrorWithSource;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(src)
    }
}

impl Circuit<FieldElement> {
    pub fn from_str(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        FromStr::from_str(src)
    }

    pub fn from_str_impl(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        let mut parser =
            Parser::new(src).map_err(|err| AcirParserErrorWithSource::parse_error(err, src))?;
        parser.parse_acir().map_err(|err| AcirParserErrorWithSource::parse_error(err, src))
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    token: SpannedToken,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> ParseResult<Self> {
        let lexer = Lexer::new(source);
        let mut parser = Self { lexer, token: eof_spanned_token() };
        parser.token = parser.read_token_internal()?;
        Ok(parser)
    }

    pub(crate) fn parse_acir(&mut self) -> ParseResult<Circuit<FieldElement>> {
        let mut circuit = Circuit::default();

        circuit.current_witness_index = self.parse_current_witness_index()?;
        circuit.private_parameters = self.parse_private_parameters()?;
        circuit.public_parameters = PublicInputs(self.parse_public_parameters()?);
        circuit.return_values = PublicInputs(self.parse_return_values()?);
        
        circuit.opcodes = self.parse_opcodes()?;

        Ok(circuit)
    }

    fn parse_current_witness_index(&mut self) -> ParseResult<u32> {
        self.eat_keyword_or_error(Keyword::Current)?;
        self.eat_keyword_or_error(Keyword::Witness)?;
        self.eat_keyword_or_error(Keyword::Index)?;
        self.eat_or_error(Token::Colon)?;

        Ok(self.eat_witness_or_error()?.0)
    }

    fn parse_private_parameters(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Private)?;
        self.eat_keyword_or_error(Keyword::Parameters)?;
        self.eat_keyword_or_error(Keyword::Indices)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_public_parameters(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Public)?;
        self.eat_keyword_or_error(Keyword::Parameters)?;
        self.eat_keyword_or_error(Keyword::Indices)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_return_values(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Return)?;
        self.eat_keyword_or_error(Keyword::Value)?;
        self.eat_keyword_or_error(Keyword::Indices)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_witness_ordered_set(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_or_error(Token::LeftBracket)?;

        let mut witnesses = BTreeSet::new();

        while !self.eat(Token::RightBracket)? {
            let witness = self.eat_witness_or_error()?;
            witnesses.insert(witness);

            // Eat optional comma
            if self.eat(Token::Comma)? {
                continue;
            }

            // If no comma, expect closing bracket next
            if self.token.token() != &Token::RightBracket {
                return self.expected_token(Token::RightBracket);
            }
        }

        Ok(witnesses)
    }

    fn parse_witness_vector(&mut self) -> ParseResult<Vec<Witness>> {
        self.parse_witness_ordered_set().map(|set| set.into_iter().collect::<Vec<_>>())
    }

    fn parse_opcodes(&mut self) -> ParseResult<Vec<Opcode<FieldElement>>> {
        let mut opcodes = Vec::new();

        while let Some(keyword) = self.peek_keyword()? {
            match keyword {
                Keyword::Expression => {
                    let expr = self.parse_arithmetic_expression()?;
                    opcodes.push(Opcode::AssertZero(expr));
                }
                Keyword::BlackBoxFuncCall => {
                    let func = self.parse_blackbox_func_call()?;
                    opcodes.push(Opcode::BlackBoxFuncCall(func));
                }
                // TODO other opcodes here:
                // Keyword::MEM => ...
                _ => break,
            }
        }

        Ok(opcodes)
    }

    fn parse_arithmetic_expression(&mut self) -> ParseResult<Expression<FieldElement>> {
        self.eat_keyword_or_error(Keyword::Expression)?;
        self.eat_or_error(Token::LeftBracket)?;

        let mut linear_combinations = Vec::new();
        let mut mul_terms = Vec::new();
        let mut constant: Option<FieldElement> = None;

        while !self.eat(Token::RightBracket)? {
            match self.token.token() {
                Token::LeftParen => {
                    // Eat '('
                    self.bump()?;
                    let coeff = self.eat_field_or_error()?;
                    self.eat_or_error(Token::Comma)?;

                    let w1 = self.eat_witness_or_error()?;

                    if self.eat(Token::Comma)? {
                        // This is a mul term
                        let w2 = self.eat_witness_or_error()?;
                        self.eat_or_error(Token::RightParen)?;
                        mul_terms.push((coeff, w1, w2));
                    } else {
                        // This is a linear term
                        self.eat_or_error(Token::RightParen)?;
                        linear_combinations.push((coeff, w1));
                    }
                }
                Token::Int(_) => {
                    if constant.is_some() {
                        return Err(ParserError::DuplicatedConstantTerm {
                            found: self.token.token().clone(),
                            span: self.token.span(),
                        });
                    }
                    constant = Some(self.eat_field_or_error()?);
                }
                _ => {
                    return Err(ParserError::ExpectedOneOfTokens {
                        tokens: vec![Token::LeftParen, Token::Int(FieldElement::zero())],
                        found: self.token.token().clone(),
                        span: self.token.span(),
                    });
                }
            }
        }

        let Some(q_c) = constant else {
            return Err(ParserError::MissingConstantTerm { span: self.token.span() });
        };

        Ok(Expression { mul_terms, linear_combinations, q_c })
    }

    // TODO: Convert all assertions on input/output lengths to real errors
    fn parse_blackbox_func_call(&mut self) -> ParseResult<BlackBoxFuncCall<FieldElement>> {
        self.eat_keyword(Keyword::BlackBoxFuncCall)?;
        self.eat_or_error(Token::Colon)?;
        self.eat_or_error(Token::Colon)?;

        // Expect an identifier like "RANGE", "AND", "XOR", etc.
        let name_ident = self.eat_ident_or_error()?;

        let Some(func_name) = BlackBoxFunc::lookup(&name_ident.to_lowercase()) else {
            return Err(ParserError::ExpectedBlackBoxFuncName {
                found: self.token.token().clone(),
                span: self.token.span(),
            });
        };

        let func = match func_name {
            BlackBoxFunc::AES128Encrypt => {
                let mut inputs = self.parse_blackbox_inputs()?;

                let key =
                    inputs.split_off(inputs.len() - 16).try_into().expect("Expected 16 inputs");
                let iv =
                    inputs.split_off(inputs.len() - 16).try_into().expect("Expected 16 inputs");

                let outputs = self.parse_witness_vector()?;

                BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs }
            }
            BlackBoxFunc::AND => {
                let inputs = self.parse_blackbox_inputs()?;
                let outputs = self.parse_witness_vector()?;

                assert_eq!(inputs.len(), 2);
                assert_eq!(outputs.len(), 1);

                BlackBoxFuncCall::XOR { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::XOR => {
                let inputs = self.parse_blackbox_inputs()?;
                let outputs = self.parse_witness_vector()?;

                assert_eq!(inputs.len(), 2);
                assert_eq!(outputs.len(), 1);

                BlackBoxFuncCall::XOR { lhs: inputs[0], rhs: inputs[1], output: outputs[0] }
            }
            BlackBoxFunc::RANGE => {
                let inputs = self.parse_blackbox_inputs()?;
                let outputs = self.parse_witness_ordered_set()?;

                assert_eq!(inputs.len(), 1);
                assert!(outputs.is_empty());

                BlackBoxFuncCall::RANGE { input: inputs[0] }
            }
            BlackBoxFunc::Blake2s => {
                let inputs = self.parse_blackbox_inputs()?;
                let outputs = self.parse_witness_vector()?.try_into().expect("Expected 32 inputs");

                BlackBoxFuncCall::Blake2s { inputs, outputs }
            }
            BlackBoxFunc::Blake3 => {
                let inputs = self.parse_blackbox_inputs()?;
                let outputs = self.parse_witness_vector()?.try_into().expect("Expected 32 inputs");

                BlackBoxFuncCall::Blake3 { inputs, outputs }
            }
            BlackBoxFunc::EcdsaSecp256k1 => {
                let mut inputs = self.parse_blackbox_inputs()?;

                let hashed_message =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");
                let signature =
                    inputs.split_off(inputs.len() - 64).try_into().expect("Expected 64 inputs");
                let public_key_y =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");
                let public_key_x =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");

                let outputs = self.parse_witness_vector()?;
                assert_eq!(outputs.len(), 1);
                let output = outputs[0];

                BlackBoxFuncCall::EcdsaSecp256k1 {
                    public_key_x,
                    public_key_y,
                    signature,
                    hashed_message,
                    output,
                }
            }
            BlackBoxFunc::EcdsaSecp256r1 => {
                let mut inputs = self.parse_blackbox_inputs()?;

                let hashed_message =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");
                let signature =
                    inputs.split_off(inputs.len() - 64).try_into().expect("Expected 64 inputs");
                let public_key_y =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");
                let public_key_x =
                    inputs.split_off(inputs.len() - 32).try_into().expect("Expected 32 inputs");

                let outputs = self.parse_witness_vector()?;
                assert_eq!(outputs.len(), 1);
                let output = outputs[0];

                BlackBoxFuncCall::EcdsaSecp256r1 {
                    public_key_x,
                    public_key_y,
                    signature,
                    hashed_message,
                    output,
                }
            }
            BlackBoxFunc::MultiScalarMul => todo!(),
            BlackBoxFunc::Keccakf1600 => {
                let inputs = self.parse_blackbox_inputs()?.try_into().expect("Expected 25 inputs");
                let outputs = self.parse_witness_vector()?.try_into().expect("Expected 25 inputs");

                BlackBoxFuncCall::Keccakf1600 { inputs, outputs }
            }
            BlackBoxFunc::RecursiveAggregation => {
                todo!("Need to change the format to dictate the size of each of input")
            }
            BlackBoxFunc::EmbeddedCurveAdd => {
                let mut inputs = self.parse_blackbox_inputs()?;

                let input2 =
                    inputs.split_off(inputs.len() - 3).try_into().expect("Expected 3 inputs");
                let input1 =
                    inputs.split_off(inputs.len() - 3).try_into().expect("Expected 3 inputs");

                let outputs = self.parse_witness_vector()?;
                assert_eq!(outputs.len(), 3);

                BlackBoxFuncCall::EmbeddedCurveAdd {
                    input1,
                    input2,
                    outputs: (outputs[0], outputs[1], outputs[2]),
                }
            }
            BlackBoxFunc::Poseidon2Permutation => {
                let inputs = self.parse_blackbox_inputs()?;
                let len = inputs.len() as u32;

                let outputs = self.parse_witness_vector()?;

                BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, len }
            }
            BlackBoxFunc::Sha256Compression => {
                let mut all_inputs = self.parse_blackbox_inputs()?;

                let hash_values = all_inputs
                    .split_off(all_inputs.len() - 8)
                    .try_into()
                    .expect("Expected 8 inputs");
                let inputs = all_inputs
                    .split_off(all_inputs.len() - 16)
                    .try_into()
                    .expect("Expected 16 inputs");

                let outputs = self.parse_witness_vector()?.try_into().expect("Expected 8 inputs");
                BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs }
            }
            BlackBoxFunc::BigIntAdd
            | BlackBoxFunc::BigIntSub
            | BlackBoxFunc::BigIntMul
            | BlackBoxFunc::BigIntDiv
            | BlackBoxFunc::BigIntFromLeBytes
            | BlackBoxFunc::BigIntToLeBytes => {
                unreachable!("Set to be removed, thus they are not implemented")
            }
        };
        Ok(func)
    }

    fn parse_blackbox_inputs(&mut self) -> ParseResult<Vec<FunctionInput<FieldElement>>> {
        self.eat_or_error(Token::LeftBracket)?;

        let mut inputs = Vec::new();

        while !self.eat(Token::RightBracket)? {
            self.eat_or_error(Token::LeftParen)?;

            let input = match self.token.token() {
                Token::Int(value) => {
                    let value = *value;
                    self.bump()?;
                    ConstantOrWitnessEnum::Constant(value)
                }
                Token::Witness(index) => {
                    let witness = *index;
                    self.bump()?;
                    ConstantOrWitnessEnum::Witness(Witness(witness))
                }
                other => {
                    return Err(ParserError::ExpectedOneOfTokens {
                        tokens: vec![Token::Int(FieldElement::zero()), Token::Witness(0)],
                        found: other.clone(),
                        span: self.token.span(),
                    });
                }
            };

            self.eat_or_error(Token::Comma)?;

            // Technically this is a u32 and will cause issues if we have fields smaller than u32
            // For the parser's simplicity we treat all integers as field elements.
            // It may be worth adding an integer type to the ACIR format as to distinguish between integer types and the native field.
            let num_bits = self.eat_field_or_error()?;
            let num_bits: u32 = num_bits
                .try_to_u32()
                .ok_or(ParserError::BitSizeDoesNotFit { num_bits, span: self.token.span() })?;

            let function_input = match input {
                ConstantOrWitnessEnum::Constant(value) => FunctionInput::constant(value, num_bits)
                    .map_err(|err| ParserError::InvalidInputBitSize {
                        value: err.value,
                        value_num_bits: err.value_num_bits,
                        max_bits: err.max_bits,
                        span: self.token.span(),
                    })?,
                ConstantOrWitnessEnum::Witness(witness) => {
                    FunctionInput::witness(witness, num_bits)
                }
            };

            inputs.push(function_input);

            self.eat_or_error(Token::RightParen)?;

            // Eat a comma if there is another input, but do not error if there is no comma
            // as this means we have reached the end of the inputs.
            self.eat(Token::Comma)?;
        }

        Ok(inputs)
    }

    fn eat_ident_or_error(&mut self) -> ParseResult<String> {
        if let Some(identifier) = self.eat_ident()? {
            Ok(identifier)
        } else {
            self.expected_identifier()
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

    fn eat_field_element(&mut self) -> ParseResult<Option<FieldElement>> {
        if let Token::Int(_) = self.token.token() {
            let token = self.bump()?;
            if let Token::Int(int) = token.into_token() { Ok(Some(int)) } else { unreachable!() }
        } else {
            Ok(None)
        }
    }

    fn eat_field_or_error(&mut self) -> ParseResult<FieldElement> {
        if let Some(int) = self.eat_field_element()? {
            Ok(int)
        } else {
            self.expected_field_element()
        }
    }

    fn bump(&mut self) -> ParseResult<SpannedToken> {
        let token = self.read_token_internal()?;
        Ok(std::mem::replace(&mut self.token, token))
    }

    fn read_token_internal(&mut self) -> ParseResult<SpannedToken> {
        self.lexer.next_token().map_err(ParserError::LexerError)
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

    fn eat_keyword_or_error(&mut self, keyword: Keyword) -> ParseResult<()> {
        if !self.eat_keyword(keyword)? {
            return self.expected_token(Token::Keyword(keyword));
        }
        Ok(())
    }

    fn peek_keyword(&self) -> ParseResult<Option<Keyword>> {
        match self.token.token() {
            Token::Keyword(kw) => Ok(Some(*kw)),
            _ => Ok(None),
        }
    }

    fn eat_witness(&mut self) -> ParseResult<Option<Witness>> {
        let is_witness_type = matches!(self.token.token(), Token::Witness(_));
        if is_witness_type {
            let token = self.bump()?;
            match token.into_token() {
                Token::Witness(witness) => Ok(Some(Witness(witness))),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn eat_witness_or_error(&mut self) -> ParseResult<Witness> {
        if let Some(int) = self.eat_witness()? { Ok(int) } else { self.expected_witness() }
    }

    fn eat_or_error(&mut self, token: Token) -> ParseResult<()> {
        if self.eat(token.clone())? { Ok(()) } else { self.expected_token(token) }
    }

    fn eat(&mut self, token: Token) -> ParseResult<bool> {
        if self.token.token() == &token {
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expected_identifier<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedIdentifier {
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }

    fn expected_field_element<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedFieldElement {
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }

    fn expected_witness<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedWitness {
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }

    fn expected_token<T>(&mut self, token: Token) -> ParseResult<T> {
        Err(ParserError::ExpectedToken {
            token,
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }
}

fn eof_spanned_token() -> SpannedToken {
    SpannedToken::new(Token::Eof, Default::default())
}

type ParseResult<T> = Result<T, ParserError>;

#[derive(Debug, Error)]
pub(crate) enum ParserError {
    #[error("{0}")]
    LexerError(LexerError),
    #[error("Expected '{token}', found '{found}'")]
    ExpectedToken { token: Token, found: Token, span: Span },
    #[error("Expected one of {tokens:?}, found {found}")]
    ExpectedOneOfTokens { tokens: Vec<Token>, found: Token, span: Span },
    #[error("Expected an identifier, found '{found}'")]
    ExpectedIdentifier { found: Token, span: Span },
    #[error("Expected a field element, found '{found}'")]
    ExpectedFieldElement { found: Token, span: Span },
    #[error("Expected a witness index, found '{found}'")]
    ExpectedWitness { found: Token, span: Span },
    #[error("Duplicate constant term in native Expression")]
    DuplicatedConstantTerm { found: Token, span: Span },
    #[error("Missing constant term in native Expression")]
    MissingConstantTerm { span: Span },
    #[error("Expected valid black box function name, found '{found}'")]
    ExpectedBlackBoxFuncName { found: Token, span: Span },
    #[error("Bit size must fit in u32, got: '{num_bits}'")]
    BitSizeDoesNotFit { num_bits: FieldElement, span: Span },
    #[error(
        "FunctionInput value has too many bits: value: {value}, {value_num_bits} >= {max_bits}"
    )]
    InvalidInputBitSize { value: String, value_num_bits: u32, max_bits: u32, span: Span },
}

impl ParserError {
    fn span(&self) -> Span {
        use ParserError::*;
        match self {
            LexerError(e) => e.span(),
            ExpectedToken { span, .. } => *span,
            ExpectedOneOfTokens { span, .. } => *span,
            ExpectedIdentifier { span, .. } => *span,
            ExpectedFieldElement { span, .. } => *span,
            ExpectedWitness { span, .. } => *span,
            DuplicatedConstantTerm { span, .. } => *span,
            MissingConstantTerm { span } => *span,
            ExpectedBlackBoxFuncName { span, .. } => *span,
            BitSizeDoesNotFit { span, .. } => *span,
            InvalidInputBitSize { span, .. } => *span,
        }
    }
}
