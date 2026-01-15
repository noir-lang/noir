use std::{collections::BTreeSet, str::FromStr};

use acir_field::{AcirField, FieldElement};

use lexer::{Lexer, LexerError};
use noirc_span::Span;
use thiserror::Error;
use token::{Keyword, SpannedToken, Token};

use crate::{
    BlackBoxFunc,
    circuit::{
        Circuit, Opcode, Program, PublicInputs,
        brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{AcirFunctionId, BlackBoxFuncCall, BlockId, BlockType, FunctionInput, MemOp},
    },
    native_types::{Expression, Witness},
};

mod lexer;
#[cfg(test)]
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

    #[cfg(test)]
    pub(super) fn get_error(self) -> ParserError {
        self.error
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

impl FromStr for Program<FieldElement> {
    type Err = AcirParserErrorWithSource;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(src)
    }
}

impl Program<FieldElement> {
    /// Creates a [Program] object from the given string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        FromStr::from_str(src)
    }

    pub fn from_str_impl(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        let mut parser =
            Parser::new(src).map_err(|err| AcirParserErrorWithSource::parse_error(err, src))?;
        parser.parse_program().map_err(|err| AcirParserErrorWithSource::parse_error(err, src))
    }
}

impl FromStr for Circuit<FieldElement> {
    type Err = AcirParserErrorWithSource;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(src)
    }
}

impl Circuit<FieldElement> {
    /// Creates a [Circuit] object from the given string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        FromStr::from_str(src)
    }

    pub fn from_str_impl(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        let mut parser =
            Parser::new(src).map_err(|err| AcirParserErrorWithSource::parse_error(err, src))?;
        parser.parse_circuit().map_err(|err| AcirParserErrorWithSource::parse_error(err, src))
    }
}

impl FromStr for Expression<FieldElement> {
    type Err = AcirParserErrorWithSource;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Self::from_str_impl(src)
    }
}

impl Expression<FieldElement> {
    /// Creates a [Expression] object from the given string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        FromStr::from_str(src)
    }

    pub fn from_str_impl(src: &str) -> Result<Self, AcirParserErrorWithSource> {
        let mut parser =
            Parser::new(src).map_err(|err| AcirParserErrorWithSource::parse_error(err, src))?;
        parser
            .parse_arithmetic_expression()
            .map_err(|err| AcirParserErrorWithSource::parse_error(err, src))
    }
}

pub fn parse_opcodes(src: &str) -> Result<Vec<Opcode<FieldElement>>, AcirParserErrorWithSource> {
    let mut parser =
        Parser::new(src).map_err(|err| AcirParserErrorWithSource::parse_error(err, src))?;
    parser.parse_opcodes().map_err(|err| AcirParserErrorWithSource::parse_error(err, src))
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    token: SpannedToken,
    max_witness_index: u32,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> ParseResult<Self> {
        let lexer = Lexer::new(source);
        let mut parser = Self { lexer, token: eof_spanned_token(), max_witness_index: 0 };
        parser.token = parser.read_token_internal()?;
        Ok(parser)
    }

    /// Parse multiple [Circuit] blocks and return a [Program].
    fn parse_program(&mut self) -> ParseResult<Program<FieldElement>> {
        let mut functions: Vec<Circuit<FieldElement>> = Vec::new();

        // We expect top-level "func" keywords for each circuit
        while self.eat_keyword(Keyword::Function)? {
            let func_id = self.eat_u32_or_error()?;
            let expected_id = functions.len() as u32;
            if func_id != expected_id {
                return Err(ParserError::UnexpectedFunctionId {
                    expected: expected_id,
                    found: func_id,
                    span: self.token.span(),
                });
            }

            let circuit = self.parse_circuit()?;

            functions.push(circuit);
        }

        // We don't support parsing unconstrained Brillig bytecode blocks
        let unconstrained_functions = Vec::new();
        Ok(Program { functions, unconstrained_functions })
    }

    pub(crate) fn parse_circuit(&mut self) -> ParseResult<Circuit<FieldElement>> {
        self.max_witness_index = 0;

        let private_parameters = self.parse_private_parameters()?;
        let public_parameters = PublicInputs(self.parse_public_parameters()?);
        let return_values = PublicInputs(self.parse_return_values()?);

        let opcodes = self.parse_opcodes()?;

        Ok(Circuit {
            current_witness_index: self.max_witness_index,
            opcodes,
            private_parameters,
            public_parameters,
            return_values,
            ..Default::default()
        })
    }

    fn parse_private_parameters(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Private)?;
        self.eat_keyword_or_error(Keyword::Parameters)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_public_parameters(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Public)?;
        self.eat_keyword_or_error(Keyword::Parameters)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_return_values(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.eat_keyword_or_error(Keyword::Return)?;
        self.eat_keyword_or_error(Keyword::Values)?;
        self.eat_or_error(Token::Colon)?;

        self.parse_witness_ordered_set()
    }

    fn parse_witness_vector(&mut self) -> ParseResult<Vec<Witness>> {
        self.parse_bracketed_list(|parser| parser.eat_witness_or_error())
    }

    fn parse_witness_ordered_set(&mut self) -> ParseResult<BTreeSet<Witness>> {
        self.parse_witness_vector().map(|vec| vec.into_iter().collect::<BTreeSet<_>>())
    }

    fn parse_opcodes(&mut self) -> ParseResult<Vec<Opcode<FieldElement>>> {
        let mut opcodes = Vec::new();

        while let Some(keyword) = self.peek_keyword() {
            match keyword {
                Keyword::Assert => {
                    let expr = self.parse_assert_zero_expression()?;
                    opcodes.push(Opcode::AssertZero(expr));
                }
                Keyword::BlackBoxFuncCall => {
                    opcodes.push(Opcode::BlackBoxFuncCall(self.parse_blackbox_func_call()?));
                }
                Keyword::MemoryInit => {
                    opcodes.push(self.parse_memory_init()?);
                }
                Keyword::MemoryRead => {
                    opcodes.push(self.parse_memory_read()?);
                }
                Keyword::MemoryWrite => {
                    opcodes.push(self.parse_memory_write()?);
                }
                Keyword::Brillig => {
                    opcodes.push(self.parse_brillig_call()?);
                }
                Keyword::Call => {
                    opcodes.push(self.parse_call()?);
                }
                _ => break,
            }
        }

        Ok(opcodes)
    }

    fn parse_assert_zero_expression(&mut self) -> ParseResult<Expression<FieldElement>> {
        // 'ASSERT'
        self.eat_keyword_or_error(Keyword::Assert)?;

        // Parse the left-hand side terms
        let lhs_terms = self.parse_terms_or_error()?;

        // '='
        self.eat_or_error(Token::Equal)?;

        // Parse the right-hand side terms
        let rhs_terms = self.parse_terms_or_error()?;

        // If we have something like `0 = ...` or `... = 0`, just consider the expressions
        // on the non-zero side. Otherwise we could be "moving" terms to the other side and
        // negating them, which won't accurately reflect the original expression.
        let expression = if is_zero_term(&lhs_terms) {
            build_expression_from_terms(rhs_terms.into_iter())
        } else if is_zero_term(&rhs_terms) {
            build_expression_from_terms(lhs_terms.into_iter())
        } else {
            // "Move" the terms to the left by negating them
            let rhs_terms = rhs_terms.into_iter().map(|term| term.negate()).collect::<Vec<_>>();
            build_expression_from_terms(lhs_terms.into_iter().chain(rhs_terms))
        };

        Ok(expression)
    }

    fn parse_terms_or_error(&mut self) -> ParseResult<Vec<Term>> {
        let mut terms = Vec::new();
        let mut negative = self.eat(Token::Minus)?;
        loop {
            let term = self.parse_term_or_error()?;
            let term = if negative { term.negate() } else { term };
            terms.push(term);

            if self.eat(Token::Plus)? {
                negative = false;
                continue;
            }

            if self.eat(Token::Minus)? {
                negative = true;
                continue;
            }

            break;
        }
        Ok(terms)
    }

    fn parse_term_or_error(&mut self) -> ParseResult<Term> {
        if let Some(coefficient) = self.eat_field_element()? {
            if self.eat(Token::Star)? {
                let w1 = self.eat_witness_or_error()?;
                self.parse_linear_or_multiplication_term(coefficient, w1)
            } else {
                Ok(Term::Constant(coefficient))
            }
        } else if let Some(w1) = self.eat_witness()? {
            self.parse_linear_or_multiplication_term(FieldElement::one(), w1)
        } else {
            self.expected_term()
        }
    }

    fn parse_linear_or_multiplication_term(
        &mut self,
        coefficient: FieldElement,
        w1: Witness,
    ) -> Result<Term, ParserError> {
        if self.eat(Token::Star)? {
            let w2 = self.eat_witness_or_error()?;
            Ok(Term::Multiplication(coefficient, w1, w2))
        } else {
            Ok(Term::Linear(coefficient, w1))
        }
    }

    fn parse_arithmetic_expression(&mut self) -> ParseResult<Expression<FieldElement>> {
        let terms = self.parse_terms_or_error()?;
        let expression = build_expression_from_terms(terms.into_iter());
        Ok(expression)
    }

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
                let inputs = self.parse_blackbox_inputs(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let iv = self.parse_blackbox_inputs_array::<16>(Keyword::Iv)?;
                self.eat_comma_or_error()?;

                let key = self.parse_blackbox_inputs_array::<16>(Keyword::Key)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs()?;

                BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs }
            }
            BlackBoxFunc::AND => {
                let lhs = self.parse_blackbox_input(Keyword::Lhs)?;
                self.eat_comma_or_error()?;

                let rhs = self.parse_blackbox_input(Keyword::Rhs)?;
                self.eat_comma_or_error()?;

                let output = self.parse_blackbox_output()?;
                self.eat_comma_or_error()?;

                let num_bits = self.parse_blackbox_u32(Keyword::Bits)?;

                BlackBoxFuncCall::AND { lhs, rhs, num_bits, output }
            }
            BlackBoxFunc::XOR => {
                let lhs = self.parse_blackbox_input(Keyword::Lhs)?;
                self.eat_comma_or_error()?;

                let rhs = self.parse_blackbox_input(Keyword::Rhs)?;
                self.eat_comma_or_error()?;

                let output = self.parse_blackbox_output()?;
                self.eat_comma_or_error()?;

                let num_bits = self.parse_blackbox_u32(Keyword::Bits)?;

                BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output }
            }
            BlackBoxFunc::RANGE => {
                let input = self.parse_blackbox_input(Keyword::Input)?;
                self.eat_comma_or_error()?;

                let num_bits = self.parse_blackbox_u32(Keyword::Bits)?;

                BlackBoxFuncCall::RANGE { input, num_bits }
            }
            BlackBoxFunc::Blake2s => {
                let inputs = self.parse_blackbox_inputs(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<32>()?;

                BlackBoxFuncCall::Blake2s { inputs, outputs }
            }
            BlackBoxFunc::Blake3 => {
                let inputs = self.parse_blackbox_inputs(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<32>()?;

                BlackBoxFuncCall::Blake3 { inputs, outputs }
            }
            BlackBoxFunc::EcdsaSecp256k1 => {
                let public_key_x = self.parse_blackbox_inputs_array::<32>(Keyword::PublicKeyX)?;
                self.eat_comma_or_error()?;

                let public_key_y = self.parse_blackbox_inputs_array::<32>(Keyword::PublicKeyY)?;
                self.eat_comma_or_error()?;

                let signature = self.parse_blackbox_inputs_array::<64>(Keyword::Signature)?;
                self.eat_comma_or_error()?;

                let hashed_message =
                    self.parse_blackbox_inputs_array::<32>(Keyword::HashedMessage)?;
                self.eat_comma_or_error()?;

                let predicate = self.parse_blackbox_input(Keyword::Predicate)?;
                self.eat_comma_or_error()?;

                let output = self.parse_blackbox_output()?;

                BlackBoxFuncCall::EcdsaSecp256k1 {
                    public_key_x,
                    public_key_y,
                    signature,
                    hashed_message,
                    output,
                    predicate,
                }
            }
            BlackBoxFunc::EcdsaSecp256r1 => {
                let public_key_x = self.parse_blackbox_inputs_array::<32>(Keyword::PublicKeyX)?;
                self.eat_comma_or_error()?;

                let public_key_y = self.parse_blackbox_inputs_array::<32>(Keyword::PublicKeyY)?;
                self.eat_comma_or_error()?;

                let signature = self.parse_blackbox_inputs_array::<64>(Keyword::Signature)?;
                self.eat_comma_or_error()?;

                let hashed_message =
                    self.parse_blackbox_inputs_array::<32>(Keyword::HashedMessage)?;
                self.eat_comma_or_error()?;

                let predicate = self.parse_blackbox_input(Keyword::Predicate)?;
                self.eat_comma_or_error()?;

                let output = self.parse_blackbox_output()?;

                BlackBoxFuncCall::EcdsaSecp256r1 {
                    public_key_x,
                    public_key_y,
                    signature,
                    hashed_message,
                    output,
                    predicate,
                }
            }
            BlackBoxFunc::MultiScalarMul => {
                let points = self.parse_blackbox_inputs(Keyword::Points)?;
                self.eat_comma_or_error()?;

                let scalars = self.parse_blackbox_inputs(Keyword::Scalars)?;
                self.eat_comma_or_error()?;

                let predicate = self.parse_blackbox_input(Keyword::Predicate)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<3>()?;
                let outputs = (outputs[0], outputs[1], outputs[2]);

                BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs }
            }
            BlackBoxFunc::Keccakf1600 => {
                let inputs = self.parse_blackbox_inputs_array::<25>(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<25>()?;

                BlackBoxFuncCall::Keccakf1600 { inputs, outputs }
            }
            BlackBoxFunc::RecursiveAggregation => {
                let verification_key = self.parse_blackbox_inputs(Keyword::VerificationKey)?;
                self.eat_comma_or_error()?;

                let proof = self.parse_blackbox_inputs(Keyword::Proof)?;
                self.eat_comma_or_error()?;

                let public_inputs = self.parse_blackbox_inputs(Keyword::PublicInputs)?;
                self.eat_comma_or_error()?;

                let key_hash = self.parse_blackbox_input(Keyword::KeyHash)?;
                self.eat_comma_or_error()?;

                let proof_type = self.parse_blackbox_u32(Keyword::ProofType)?;
                self.eat_comma_or_error()?;

                let predicate = self.parse_blackbox_input(Keyword::Predicate)?;
                self.eat_comma_or_error()?;

                let output = self.parse_blackbox_outputs()?;

                BlackBoxFuncCall::RecursiveAggregation {
                    verification_key,
                    proof,
                    public_inputs,
                    key_hash,
                    proof_type,
                    predicate,
                    output,
                }
            }
            BlackBoxFunc::EmbeddedCurveAdd => {
                let input1 = self.parse_blackbox_inputs_array::<3>(Keyword::Input1)?;
                self.eat_comma_or_error()?;

                let input2 = self.parse_blackbox_inputs_array::<3>(Keyword::Input2)?;
                self.eat_comma_or_error()?;

                let predicate = self.parse_blackbox_input(Keyword::Predicate)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<3>()?;
                let outputs = (outputs[0], outputs[1], outputs[2]);

                BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs }
            }
            BlackBoxFunc::Poseidon2Permutation => {
                let inputs = self.parse_blackbox_inputs(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs()?;

                BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs }
            }
            BlackBoxFunc::Sha256Compression => {
                let inputs = self.parse_blackbox_inputs_array::<16>(Keyword::Inputs)?;
                self.eat_comma_or_error()?;

                let hash_values = self.parse_blackbox_inputs_array::<8>(Keyword::HashValues)?;
                self.eat_comma_or_error()?;

                let outputs = self.parse_blackbox_outputs_array::<8>()?;

                BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs }
            }
        };
        Ok(func)
    }

    fn parse_blackbox_inputs_array<const N: usize>(
        &mut self,
        keyword: Keyword,
    ) -> Result<Box<[FunctionInput<FieldElement>; N]>, ParserError> {
        let inputs = self.parse_blackbox_inputs(keyword)?;
        self.try_vec_to_array::<N, _>(inputs, keyword)
    }

    fn parse_blackbox_inputs(
        &mut self,
        keyword: Keyword,
    ) -> ParseResult<Vec<FunctionInput<FieldElement>>> {
        self.eat_keyword_or_error(keyword)?;
        self.eat_or_error(Token::Colon)?;
        self.parse_bracketed_list(|parser| parser.parse_blackbox_input_no_keyword())
    }

    fn parse_blackbox_input(
        &mut self,
        keyword: Keyword,
    ) -> Result<FunctionInput<FieldElement>, ParserError> {
        self.eat_keyword_or_error(keyword)?;
        self.eat_or_error(Token::Colon)?;
        self.parse_blackbox_input_no_keyword()
    }

    fn parse_blackbox_input_no_keyword(
        &mut self,
    ) -> Result<FunctionInput<FieldElement>, ParserError> {
        if let Some(value) = self.eat_field_element()? {
            Ok(FunctionInput::Constant(value))
        } else if let Some(witness) = self.eat_witness()? {
            Ok(FunctionInput::Witness(witness))
        } else {
            Err(ParserError::ExpectedOneOfTokens {
                tokens: vec![Token::Int(FieldElement::zero()), Token::Witness(0)],
                found: self.token.token().clone(),
                span: self.token.span(),
            })
        }
    }

    fn parse_blackbox_output(&mut self) -> ParseResult<Witness> {
        self.eat_keyword_or_error(Keyword::Output)?;
        self.eat_or_error(Token::Colon)?;
        let witness = self.eat_witness_or_error()?;
        Ok(witness)
    }

    fn parse_blackbox_outputs_array<const N: usize>(&mut self) -> ParseResult<Box<[Witness; N]>> {
        let outputs = self.parse_blackbox_outputs()?;
        self.try_vec_to_array::<N, _>(outputs, Keyword::Outputs)
    }

    fn parse_blackbox_outputs(&mut self) -> ParseResult<Vec<Witness>> {
        self.eat_keyword_or_error(Keyword::Outputs)?;
        self.eat_or_error(Token::Colon)?;
        self.parse_witness_vector()
    }

    fn parse_blackbox_u32(&mut self, keyword: Keyword) -> ParseResult<u32> {
        self.eat_keyword_or_error(keyword)?;
        self.eat_or_error(Token::Colon)?;
        let num_bits = self.eat_u32_or_error()?;

        Ok(num_bits)
    }

    fn parse_memory_init(&mut self) -> ParseResult<Opcode<FieldElement>> {
        self.eat_keyword_or_error(Keyword::MemoryInit)?;

        let block_type = self.parse_block_type()?;

        // blockId = [witness1, witness2, ...]
        let block_id = self.eat_block_id_or_error()?;
        self.eat_or_error(Token::Equal)?;
        let init = self.parse_witness_vector()?;

        Ok(Opcode::MemoryInit { block_id, init, block_type })
    }

    fn parse_block_type(&mut self) -> Result<BlockType, ParserError> {
        if self.eat_keyword(Keyword::CallData)? {
            Ok(BlockType::CallData(self.eat_u32_or_error()?))
        } else if self.eat_keyword(Keyword::ReturnData)? {
            Ok(BlockType::ReturnData)
        } else {
            Ok(BlockType::Memory)
        }
    }

    fn parse_memory_read(&mut self) -> ParseResult<Opcode<FieldElement>> {
        self.eat_keyword_or_error(Keyword::MemoryRead)?;

        // value = blockId[index]
        let value = self.parse_arithmetic_expression()?;
        self.eat_or_error(Token::Equal)?;
        let block_id = self.eat_block_id_or_error()?;
        self.eat_or_error(Token::LeftBracket)?;
        let index = self.parse_arithmetic_expression()?;
        self.eat_or_error(Token::RightBracket)?;

        let operation = Expression::zero();

        Ok(Opcode::MemoryOp { block_id, op: MemOp { index, value, operation } })
    }

    fn parse_memory_write(&mut self) -> ParseResult<Opcode<FieldElement>> {
        self.eat_keyword_or_error(Keyword::MemoryWrite)?;

        // blockId[index] = value
        let block_id = self.eat_block_id_or_error()?;
        self.eat_or_error(Token::LeftBracket)?;
        let index = self.parse_arithmetic_expression()?;
        self.eat_or_error(Token::RightBracket)?;
        self.eat_or_error(Token::Equal)?;
        let value = self.parse_arithmetic_expression()?;

        let operation = Expression::one();

        Ok(Opcode::MemoryOp { block_id, op: MemOp { index, value, operation } })
    }

    fn parse_brillig_call(&mut self) -> ParseResult<Opcode<FieldElement>> {
        self.eat_keyword_or_error(Keyword::Brillig)?;
        self.eat_keyword_or_error(Keyword::Call)?;
        self.eat_keyword_or_error(Keyword::Function)?;
        self.eat_or_error(Token::Colon)?;
        let func_id = self.eat_u32_or_error()?;
        self.eat_or_error(Token::Comma)?;

        let predicate = self.eat_predicate()?;

        // Parse inputs
        self.eat_keyword_or_error(Keyword::Inputs)?;
        self.eat_or_error(Token::Colon)?;
        let inputs = self.parse_brillig_inputs()?;

        self.eat_or_error(Token::Comma)?; // between inputs and outputs

        // Parse outputs
        self.eat_keyword_or_error(Keyword::Outputs)?;
        self.eat_or_error(Token::Colon)?;
        let outputs = self.parse_brillig_outputs()?;

        Ok(Opcode::BrilligCall { id: BrilligFunctionId(func_id), inputs, outputs, predicate })
    }

    fn parse_brillig_inputs(&mut self) -> ParseResult<Vec<BrilligInputs<FieldElement>>> {
        self.parse_bracketed_list(|parser| parser.parse_brillig_input())
    }

    fn parse_brillig_input(&mut self) -> Result<BrilligInputs<FieldElement>, ParserError> {
        if self.at(Token::LeftBracket) {
            // It's an array of expressions
            let exprs = self.parse_bracketed_list(|parser| parser.parse_arithmetic_expression())?;
            Ok(BrilligInputs::Array(exprs))
        } else if let Some(block_id) = self.eat_block_id()? {
            Ok(BrilligInputs::MemoryArray(block_id))
        } else {
            let expr = self.parse_arithmetic_expression()?;
            Ok(BrilligInputs::Single(expr))
        }
    }

    fn parse_brillig_outputs(&mut self) -> ParseResult<Vec<BrilligOutputs>> {
        self.parse_bracketed_list(|parser| parser.parse_brillig_output())
    }

    fn parse_brillig_output(&mut self) -> Result<BrilligOutputs, ParserError> {
        if self.at(Token::LeftBracket) {
            let witnesses = self.parse_witness_vector()?;
            Ok(BrilligOutputs::Array(witnesses))
        } else if let Some(witness) = self.eat_witness()? {
            Ok(BrilligOutputs::Simple(witness))
        } else {
            self.expected_one_of_tokens(&[Token::LeftBracket, Token::Witness(0)])
        }
    }

    fn parse_call(&mut self) -> ParseResult<Opcode<FieldElement>> {
        self.eat_keyword_or_error(Keyword::Call)?;
        self.eat_keyword_or_error(Keyword::Function)?;
        self.eat_or_error(Token::Colon)?;
        let id = self.eat_u32_or_error()?;
        self.eat_or_error(Token::Comma)?;
        let predicate = self.eat_predicate()?;

        self.eat_keyword_or_error(Keyword::Inputs)?;
        self.eat_or_error(Token::Colon)?;
        let inputs = self.parse_witness_vector()?;

        self.eat_or_error(Token::Comma)?;
        self.eat_keyword_or_error(Keyword::Outputs)?;
        self.eat_or_error(Token::Colon)?;
        let outputs = self.parse_witness_vector()?;

        Ok(Opcode::Call { id: AcirFunctionId(id), inputs, outputs, predicate })
    }

    fn eat_predicate(&mut self) -> ParseResult<Option<Expression<FieldElement>>> {
        let mut predicate = None;
        if self.eat_keyword(Keyword::Predicate)? && self.eat(Token::Colon)? {
            let expr = self.parse_arithmetic_expression()?;
            self.eat_or_error(Token::Comma)?;
            predicate = Some(expr);
        }
        Ok(predicate)
    }

    fn parse_bracketed_list<T, F>(&mut self, parser: F) -> ParseResult<Vec<T>>
    where
        F: Fn(&mut Parser<'a>) -> ParseResult<T>,
    {
        self.eat_or_error(Token::LeftBracket)?;

        let mut values = Vec::new();

        while !self.eat(Token::RightBracket)? {
            let value = parser(self)?;
            values.push(value);

            // Eat optional comma
            if self.eat(Token::Comma)? {
                continue;
            }

            // If no comma, expect closing bracket next
            self.eat_or_error(Token::RightBracket)?;
            break;
        }

        Ok(values)
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

    fn eat_u32_or_error(&mut self) -> ParseResult<u32> {
        // u32s will cause issues if we have fields smaller than u32.
        // For the parser's simplicity we treat all integers as field elements.
        // It may be worth adding an integer type to the ACIR format as to distinguish between integer types and the native field.
        let number = self.eat_field_or_error()?;
        number
            .try_to_u32()
            .ok_or(ParserError::IntegerLargerThanU32 { number, span: self.token.span() })
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

    fn peek_keyword(&self) -> Option<Keyword> {
        match self.token.token() {
            Token::Keyword(kw) => Some(*kw),
            _ => None,
        }
    }

    fn eat_witness(&mut self) -> ParseResult<Option<Witness>> {
        let is_witness_type = matches!(self.token.token(), Token::Witness(_));
        if is_witness_type {
            let token = self.bump()?;
            match token.into_token() {
                Token::Witness(witness) => {
                    if witness > self.max_witness_index {
                        self.max_witness_index = witness;
                    }
                    Ok(Some(Witness(witness)))
                }
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn eat_witness_or_error(&mut self) -> ParseResult<Witness> {
        if let Some(int) = self.eat_witness()? { Ok(int) } else { self.expected_witness() }
    }

    fn eat_block_id(&mut self) -> ParseResult<Option<BlockId>> {
        let is_block_type = matches!(self.token.token(), Token::Block(_));
        if is_block_type {
            let token = self.bump()?;
            match token.into_token() {
                Token::Block(block) => Ok(Some(BlockId(block))),
                _ => unreachable!(),
            }
        } else {
            Ok(None)
        }
    }

    fn eat_block_id_or_error(&mut self) -> ParseResult<BlockId> {
        if let Some(int) = self.eat_block_id()? { Ok(int) } else { self.expected_block_id() }
    }

    fn eat_comma_or_error(&mut self) -> ParseResult<()> {
        self.eat_or_error(Token::Comma)
    }

    fn eat_or_error(&mut self, token: Token) -> ParseResult<()> {
        if self.eat(token.clone())? { Ok(()) } else { self.expected_token(token) }
    }

    fn at(&mut self, token: Token) -> bool {
        self.token.token() == &token
    }

    /// Returns true if the token is eaten and bumps to the next token.
    /// Otherwise will return false and no bump will occur.
    fn eat(&mut self, token: Token) -> ParseResult<bool> {
        if self.token.token() == &token {
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn try_vec_to_array<const N: usize, T: Clone>(
        &self,
        vec: Vec<T>,
        keyword: Keyword,
    ) -> Result<Box<[T; N]>, ParserError> {
        let len = vec.len();
        vec.try_into().map_err(|_| ParserError::IncorrectValuesLength {
            expected: N,
            found: len,
            name: keyword.to_string(),
            span: self.token.span(),
        })
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

    fn expected_block_id<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedBlockId {
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }

    fn expected_term<T>(&mut self) -> ParseResult<T> {
        Err(ParserError::ExpectedTerm {
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

    fn expected_one_of_tokens<T>(&mut self, tokens: &[Token]) -> ParseResult<T> {
        Err(ParserError::ExpectedOneOfTokens {
            tokens: tokens.to_vec(),
            found: self.token.token().clone(),
            span: self.token.span(),
        })
    }
}

fn build_expression_from_terms(terms: impl Iterator<Item = Term>) -> Expression<FieldElement> {
    // Gather all terms, summing the constants
    let mut q_c = FieldElement::zero();
    let mut linear_combinations = Vec::new();
    let mut mul_terms = Vec::new();

    for term in terms {
        match term {
            Term::Constant(c) => q_c += c,
            Term::Linear(c, w) => linear_combinations.push((c, w)),
            Term::Multiplication(c, w1, w2) => mul_terms.push((c, w1, w2)),
        }
    }

    Expression { mul_terms, linear_combinations, q_c }
}

fn is_zero_term(terms: &[Term]) -> bool {
    terms.len() == 1 && matches!(terms[0], Term::Constant(c) if c.is_zero())
}

fn eof_spanned_token() -> SpannedToken {
    SpannedToken::new(Token::Eof, Default::default())
}

#[derive(Debug, Clone, Copy)]
enum Term {
    Constant(FieldElement),
    Linear(FieldElement, Witness),
    Multiplication(FieldElement, Witness, Witness),
}

impl Term {
    fn negate(self) -> Self {
        match self {
            Term::Constant(c) => Term::Constant(-c),
            Term::Linear(c, w) => Term::Linear(-c, w),
            Term::Multiplication(c, w1, w2) => Term::Multiplication(-c, w1, w2),
        }
    }
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
    #[error("Expected a block ID, found '{found}'")]
    ExpectedBlockId { found: Token, span: Span },
    #[error("Expected a term, found '{found}'")]
    ExpectedTerm { found: Token, span: Span },
    #[error("Expected valid black box function name, found '{found}'")]
    ExpectedBlackBoxFuncName { found: Token, span: Span },
    #[error("Number does not fit in u32, got: '{number}'")]
    IntegerLargerThanU32 { number: FieldElement, span: Span },
    #[error("Expected {expected} values for {name}, found {found}")]
    IncorrectValuesLength { expected: usize, found: usize, name: String, span: Span },
    #[error("Expected function id {expected}, found {found}")]
    UnexpectedFunctionId { expected: u32, found: u32, span: Span },
}

impl ParserError {
    fn span(&self) -> Span {
        use ParserError::*;
        match self {
            LexerError(e) => e.span(),
            ExpectedToken { span, .. }
            | ExpectedOneOfTokens { span, .. }
            | ExpectedIdentifier { span, .. }
            | ExpectedFieldElement { span, .. }
            | ExpectedWitness { span, .. }
            | ExpectedBlockId { span, .. }
            | ExpectedTerm { span, .. }
            | ExpectedBlackBoxFuncName { span, .. }
            | IntegerLargerThanU32 { span, .. }
            | IncorrectValuesLength { span, .. }
            | UnexpectedFunctionId { span, .. } => *span,
        }
    }
}
