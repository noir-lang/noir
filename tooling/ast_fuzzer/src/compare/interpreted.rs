//! Compare an arbitrary AST compiled into SSA and executed with the
//! SSA interpreter at some stage of the SSA pipeline.

use std::sync::{Arc, OnceLock};

use arbitrary::Unstructured;
use color_eyre::eyre;
use iter_extended::vecmap;
use noirc_abi::{Abi, AbiType, InputMap, Sign, input_parser::InputValue};
use noirc_evaluator::ssa::{self, ir::types::NumericType, ssa_gen::Ssa};
use noirc_frontend::{Shared, monomorphization::ast::Program};
use regex::Regex;

use crate::{Config, arb_program, input::arb_inputs_from_ssa, program_abi};

use super::{Comparable, CompareOptions, CompareResult, ExecOutput};

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

    /// Options that influence the pipeline, common to both passes.
    pub options: CompareOptions,
    pub ssa1: ComparePass,
    pub ssa2: ComparePass,
}

impl CompareInterpreted {
    /// Inputs for the `main` function in the SSA.
    ///
    /// Can't be stored as a field because we need a fresh copy for each interpreter run.
    pub fn input_values(&self) -> Vec<ssa::interpreter::value::Value> {
        input_values_to_ssa(&self.abi, &self.input_map)
    }

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

        Ok(Self { program, abi, input_map, options, ssa1, ssa2 })
    }

    pub fn exec(&self) -> eyre::Result<CompareInterpretedResult> {
        // Debug prints up fron tin case the interpreter panics. Turn them on with `RUST_LOG=debug cargo test ...`
        log::debug!("program: \n{}\n", crate::DisplayAstAsNoir(&self.program));
        log::debug!(
            "input map: \n{}\n",
            noirc_abi::input_parser::Format::Toml.serialize(&self.input_map, &self.abi).unwrap()
        );
        log::debug!(
            "input values:\n{}\n",
            self.input_values()
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{i}: {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        log::debug!("SSA after step {} ({}):\n{}\n", self.ssa1.step, self.ssa1.msg, self.ssa1.ssa);
        let res1 = self.ssa1.ssa.interpret(self.input_values());
        let res2 = self.ssa2.ssa.interpret(self.input_values());
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

impl Comparable for ssa::interpreter::errors::InterpreterError {
    fn equivalent(e1: &Self, e2: &Self) -> bool {
        use ssa::interpreter::errors::InterpreterError::*;

        match (e1, e2) {
            (Internal(_), _) | (_, Internal(_)) => {
                // We should not get, or ignore, internal errors.
                // They mean the interpreter got something unexpected that we need to fix.
                false
            }
            (Overflow { instruction: i1 }, Overflow { instruction: i2 }) => {
                // Overflows can occur or uncomparable instructions, but in a parentheses it contains the values that caused it.
                fn details(s: &str) -> Option<&str> {
                    let start = s.find("(")?;
                    let end = s.find(")")?;
                    (start < end).then(|| &s[start..=end])
                }
                fn details_or_sanitize(s: &str) -> String {
                    details(s).map(|s| s.to_string()).unwrap_or_else(|| sanitize_ssa(s))
                }
                details_or_sanitize(i1) == details_or_sanitize(i2)
            }
            (e1, e2) => {
                // The format strings contain SSA instructions,
                // where the only difference might be the value ID.
                let s1 = format!("{e1}");
                let s2 = format!("{e2}");
                sanitize_ssa(&s1) == sanitize_ssa(&s2)
            }
        }
    }
}

impl Comparable for ssa::interpreter::value::Value {
    fn equivalent(a: &Self, b: &Self) -> bool {
        use ssa::interpreter::value::Value;
        match (a, b) {
            (Value::ArrayOrSlice(a), Value::ArrayOrSlice(b)) => {
                // Ignore the RC
                a.element_types == b.element_types
                    && Comparable::equivalent(&a.elements, &b.elements)
                    && a.is_slice == b.is_slice
            }
            (Value::Reference(a), Value::Reference(b)) => {
                // Ignore the original ID
                a.element_type == b.element_type && Comparable::equivalent(&a.element, &b.element)
            }
            (a, b) => a == b,
        }
    }
}

/// Convert the ABI encoded inputs to what the SSA interpreter expects.
pub fn input_values_to_ssa(abi: &Abi, input_map: &InputMap) -> Vec<ssa::interpreter::value::Value> {
    let mut inputs = Vec::new();
    for param in &abi.parameters {
        let input = &input_map
            .get(&param.name)
            .unwrap_or_else(|| panic!("parameter not found in input: {}", param.name));

        inputs.extend(input_value_to_ssa(&param.typ, input));
    }
    inputs
}

/// Convert one ABI encoded input to what the SSA interpreter expects.
///
/// Tuple types are returned flattened.
fn input_value_to_ssa(typ: &AbiType, input: &InputValue) -> Vec<ssa::interpreter::value::Value> {
    use ssa::interpreter::value::{ArrayValue, NumericValue, Value};
    use ssa::ir::types::Type;
    let array_value = |elements: Vec<Vec<Value>>, types: Vec<Type>| {
        let elements = elements.into_iter().flatten().collect();
        let arr = Value::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(types),
            is_slice: false,
        });
        vec![arr]
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
            let num_val = NumericValue::from_constant(*f, num_typ).expect("cannot create constant");
            vec![Value::Numeric(num_val)]
        }
        InputValue::String(s) => array_value(
            vec![vecmap(s.as_bytes(), |b| Value::Numeric(NumericValue::U8(*b)))],
            vec![Type::unsigned(8)],
        ),
        InputValue::Vec(input_values) => match typ {
            AbiType::Array { length, typ } => {
                assert_eq!(*length as usize, input_values.len(), "array length != input length");
                let elements = vecmap(input_values, |input| input_value_to_ssa(typ, input));
                array_value(elements, vec![input_type_to_ssa(typ)])
            }
            AbiType::Tuple { fields } => {
                assert_eq!(fields.len(), input_values.len(), "tuple size != input length");

                let elements = vecmap(fields.iter().zip(input_values), |(typ, input)| {
                    input_value_to_ssa(typ, input)
                })
                .into_iter()
                .flatten()
                .collect();

                // Tuples are not wrapped into arrays, they are returned as a vector.
                elements
            }
            other => {
                panic!("unexpected ABI type for vector input: {other:?}")
            }
        },
        InputValue::Struct(field_values) => match typ {
            AbiType::Struct { path: _, fields } => {
                assert_eq!(fields.len(), field_values.len(), "struct size != input length");
                let elements = vecmap(fields, |(name, typ)| {
                    let input = &field_values[name];
                    input_value_to_ssa(typ, input)
                });
                array_value(elements, vecmap(fields.iter().map(|(_, typ)| typ), input_type_to_ssa))
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

/// Remove identifiers from the SSA, so we can compare the structure without
/// worrying about trivial differences like changing IDs of the same variable
/// between one pass to the next.
fn sanitize_ssa(ssa: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Capture function ID, value IDs, global IDs.
    let re = RE.get_or_init(|| Regex::new(r#"(f|b|v|g)\d+"#).expect("ID regex failed"));
    re.replace_all(ssa, "${1}_").into_owned()
}

#[cfg(test)]
mod tests {
    use super::sanitize_ssa;

    #[test]
    fn test_sanitize_ssa() {
        let src = r#"
        g1 = make_array [i8 114, u32 2354179802, i8 37, i8 179, u32 1465519558, i8 87] : [(i8, u32, i8); 2]

        acir(inline) fn main f0 {
        b0(v7: i8, v8: u32, v9: i8, v10: [(i8, i8, u1, u1, [u8; 0]); 2]):
            v17 = allocate -> &mut u32
            store u32 25 at v17
            v19 = cast v9 as i64
            v29 = array_get v10, index u32 9 -> [u8; 0]
            v30 = cast v23 as i64
            v31 = lt v30, v19
            v32 = not v31
            jmpif v32 then: b1, else: b2
        "#;

        let ssa = sanitize_ssa(src);

        similar_asserts::assert_eq!(
            ssa,
            r#"
        g_ = make_array [i8 114, u32 2354179802, i8 37, i8 179, u32 1465519558, i8 87] : [(i8, u32, i8); 2]

        acir(inline) fn main f_ {
        b_(v_: i8, v_: u32, v_: i8, v_: [(i8, i8, u1, u1, [u8; 0]); 2]):
            v_ = allocate -> &mut u32
            store u32 25 at v_
            v_ = cast v_ as i64
            v_ = array_get v_, index u32 9 -> [u8; 0]
            v_ = cast v_ as i64
            v_ = lt v_, v_
            v_ = not v_
            jmpif v_ then: b_, else: b_
        "#
        );
    }
}
