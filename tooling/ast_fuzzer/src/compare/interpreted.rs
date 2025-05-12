//! Compare an arbitrary AST compiled into SSA and executed with the
//! SSA interpreter at some stage of the SSA pipeline.

use std::sync::Arc;

use arbitrary::Unstructured;
use color_eyre::eyre;
use iter_extended::vecmap;
use noirc_abi::{Abi, AbiType, InputMap, Sign, input_parser::InputValue};
use noirc_evaluator::ssa::{self, ir::types::NumericType, ssa_gen::Ssa};
use noirc_frontend::{Shared, monomorphization::ast::Program};

use crate::{Config, arb_program, input::arb_inputs_from_ssa, program_abi};

use super::{CompareError, CompareOptions, CompareResult, ExecOutput};

/// The state of the SSA after a particular pass in the pipeline.
pub struct ComparePass {
    /// The overall position of this pass in the pipeline.
    ///
    /// The Initial SSA is considered step 0.
    pub step: usize,
    /// The message (without the counter) of the pass.
    pub msg: String,
    /// The state of the SSA after the pass.
    pub ssa: Ssa,
}

type InterpretResult =
    Result<Vec<ssa::interpreter::value::Value>, ssa::interpreter::errors::InterpreterError>;

/// The result of the SSA interpreter execution.
pub type CompareInterpretedResult =
    CompareResult<Vec<ssa::interpreter::value::Value>, ssa::interpreter::errors::InterpreterError>;

/// Inputs for comparing the interpretation of two SSA states of an arbitrary program.
pub struct CompareInterpreted {
    pub program: Program,
    pub abi: Abi,
    /// ABI inputs, which we map to SSA values as part of the execution.
    /// We could generate random input for the SSA directly, but it would
    /// make it more difficult to use it with `nargo` if we find a failure.
    pub input_map: InputMap,
    /// Inputs for the `main` function in the SSA.
    pub input_values: Vec<ssa::interpreter::value::Value>,
    /// Options that influence the pipeline, common to both passes.
    pub options: CompareOptions,
    pub ssa1: ComparePass,
    pub ssa2: ComparePass,
}

impl CompareInterpreted {
    /// 1. Generate an arbitrary AST
    /// 2. Stop the compilation at two arbitrary SSA passes
    /// 3. Generate input for the main function of the SSA
    pub fn arb(
        u: &mut Unstructured,
        c: Config,
        f: impl FnOnce(
            &mut Unstructured,
            Program,
        ) -> arbitrary::Result<(CompareOptions, ComparePass, ComparePass)>,
    ) -> arbitrary::Result<Self> {
        let program = arb_program(u, c)?;
        let abi = program_abi(&program);
        let (options, ssa1, ssa2) = f(u, program.clone())?;

        let input_map = arb_inputs_from_ssa(u, &ssa1.ssa, &abi)?;
        let input_values = input_values_to_ssa(&abi, &input_map);

        Ok(Self { program, abi, input_map, input_values, options, ssa1, ssa2 })
    }

    pub fn exec(&self) -> eyre::Result<CompareInterpretedResult> {
        // println!("program: \n{}\n", DisplayAstAsNoir(&self.program));
        // println!(
        //     "input map: \n{}\n",
        //     noirc_abi::input_parser::Format::Toml.serialize(&self.input_map, &self.abi).unwrap()
        // );
        // println!(
        //     "input values:\n{}\n",
        //     self.input_values
        //         .iter()
        //         .enumerate()
        //         .map(|(i, v)| format!("{i}: {v}"))
        //         .collect::<Vec<_>>()
        //         .join("\n")
        // );
        // println!("SSA:\n{}\n", self.ssa1.ssa);
        let res1 = self.ssa1.ssa.interpret(self.input_values.clone());
        let res2 = self.ssa2.ssa.interpret(self.input_values.clone());
        Ok(CompareInterpretedResult::new(res1, res2))
    }
}

impl CompareInterpretedResult {
    pub fn new(res1: InterpretResult, res2: InterpretResult) -> Self {
        let out = |ret| ExecOutput { return_value: Some(ret), print_output: Default::default() };
        match (res1, res2) {
            (Ok(r1), Ok(e2)) => Self::BothPassed(out(r1), out(e2)),
            (Ok(r1), Err(e2)) => Self::RightFailed(out(r1), e2),
            (Err(e1), Ok(r2)) => Self::LeftFailed(e1, out(r2)),
            (Err(e1), Err(e2)) => Self::BothFailed(e1, e2),
        }
    }
}

impl CompareError for ssa::interpreter::errors::InterpreterError {
    fn equivalent(e1: &Self, e2: &Self) -> bool {
        e1 == e2
    }
}

/// Convert the ABI encoded inputs to what the SSA interpreter expects.
fn input_values_to_ssa(abi: &Abi, input_map: &InputMap) -> Vec<ssa::interpreter::value::Value> {
    let mut inputs = Vec::new();
    for param in &abi.parameters {
        let input = &input_map
            .get(&param.name)
            .unwrap_or_else(|| panic!("parameter not found in input: {}", param.name));
        let input = input_value_to_ssa(&param.typ, input);
        inputs.push(input);
    }
    inputs
}

/// Convert one ABI encoded input to what the SSA interpreter expects.
fn input_value_to_ssa(typ: &AbiType, input: &InputValue) -> ssa::interpreter::value::Value {
    use ssa::interpreter::value::{ArrayValue, NumericValue, Value};
    use ssa::ir::types::Type;
    let array_value = |elements, types| {
        Value::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(types),
            is_slice: false,
        })
    };
    match input {
        InputValue::Field(f) => {
            let num_typ = match typ {
                AbiType::Field => NumericType::NativeField,
                AbiType::Boolean => NumericType::Unsigned { bit_size: 1 },
                AbiType::Integer { sign: Sign::Signed, width } => {
                    NumericType::Signed { bit_size: *width }
                }
                AbiType::Integer { sign: Sign::Unsigned, width } => {
                    NumericType::Unsigned { bit_size: *width }
                }
                other => panic!("unexpected ABY type for Field input: {other:?}"),
            };
            Value::Numeric(NumericValue::from_constant(*f, num_typ))
        }
        InputValue::String(s) => array_value(
            vecmap(s.as_bytes(), |b| Value::Numeric(NumericValue::U8(*b))),
            vec![Type::unsigned(8)],
        ),
        InputValue::Vec(input_values) => match typ {
            AbiType::Array { length, typ } => {
                assert_eq!(*length as usize, input_values.len(), "array length != input length");
                array_value(
                    vecmap(input_values, |input| input_value_to_ssa(typ, input)),
                    vec![input_type_to_ssa(typ)],
                )
            }
            AbiType::Tuple { fields } => {
                assert_eq!(fields.len(), input_values.len(), "tuple size != input length");
                array_value(
                    vecmap(fields.iter().zip(input_values), |(typ, input)| {
                        input_value_to_ssa(typ, input)
                    }),
                    vecmap(fields, input_type_to_ssa),
                )
            }
            other => {
                panic!("unexpected ABI type for vector input: {other:?}")
            }
        },
        InputValue::Struct(field_values) => match typ {
            AbiType::Struct { path: _, fields } => {
                assert_eq!(fields.len(), field_values.len(), "struct size != input length");
                array_value(
                    vecmap(fields, |(name, typ)| {
                        let input = &field_values[name];
                        input_value_to_ssa(typ, input)
                    }),
                    vecmap(fields.iter().map(|(_, typ)| typ), input_type_to_ssa),
                )
            }
            other => {
                panic!("unexpected ABI type for map input: {other:?}")
            }
        },
    }
}

/// Convert an ABI type into SSA.
fn input_type_to_ssa(typ: &AbiType) -> ssa::ir::types::Type {
    use ssa::ir::types::Type;
    match typ {
        AbiType::Field => Type::field(),
        AbiType::Array { length, typ } => {
            Type::Array(Arc::new(vec![input_type_to_ssa(typ)]), *length)
        }
        AbiType::Integer { sign: Sign::Signed, width } => Type::signed(*width),
        AbiType::Integer { sign: Sign::Unsigned, width } => Type::unsigned(*width),
        AbiType::Boolean => Type::bool(),
        AbiType::Struct { path: _, fields } => Type::Array(
            Arc::new(vecmap(fields, |(_, typ)| input_type_to_ssa(typ))),
            fields.len() as u32,
        ),
        AbiType::Tuple { fields } => {
            Type::Array(Arc::new(vecmap(fields, input_type_to_ssa)), fields.len() as u32)
        }
        AbiType::String { length } => Type::Array(Arc::new(vec![Type::unsigned(8)]), *length),
    }
}
