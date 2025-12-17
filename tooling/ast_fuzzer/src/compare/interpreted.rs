//! Compare an arbitrary AST compiled into SSA and executed with the
//! SSA interpreter at some stage of the SSA pipeline.

use std::sync::{Arc, OnceLock};

use arbitrary::Unstructured;
use color_eyre::eyre;
use iter_extended::vecmap;
use noirc_abi::{Abi, AbiType, InputMap, Sign, input_parser::InputValue};
use noirc_evaluator::ssa::{
    self,
    interpreter::{InterpreterOptions, value::Value},
    ir::{instruction::BinaryOp, types::NumericType},
    ssa_gen::Ssa,
};
use noirc_frontend::{Shared, monomorphization::ast::Program};
use regex::Regex;

use crate::{Config, arb_program, compare::logging, input::arb_inputs_from_ssa, program_abi};

use super::{Comparable, CompareOptions, CompareResult, FailedOutput, PassedOutput};

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

type InterpretResult = Result<Vec<Value>, ssa::interpreter::errors::InterpreterError>;

/// The result of the SSA interpreter execution.
pub type CompareInterpretedResult =
    CompareResult<Vec<Value>, ssa::interpreter::errors::InterpreterError>;

/// Inputs for comparing the interpretation of two SSA states of an arbitrary program.
pub struct CompareInterpreted {
    pub program: Program,
    pub abi: Abi,
    /// ABI inputs, which we map to SSA values as part of the execution.
    /// We could generate random input for the SSA directly, but it would
    /// make it more difficult to use it with `nargo` if we find a failure.
    pub input_map: InputMap,

    /// Inputs for the `main` function in the SSA, mapped from the ABI input.
    pub ssa_args: Vec<Value>,

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
        logging::log_program(&program, "");

        let (options, ssa1, ssa2) = f(u, program.clone())?;

        logging::log_options(&options, "");
        logging::log_ssa(&ssa1.ssa, &format!("after step {} - {}", ssa1.step, ssa1.msg));
        logging::log_ssa(&ssa2.ssa, &format!("after step {} - {}", ssa2.step, ssa2.msg));

        let input_map = arb_inputs_from_ssa(u, &ssa1.ssa, &abi)?;
        logging::log_abi_inputs(&abi, &input_map);

        let ssa_args = input_values_to_ssa(&abi, &input_map);
        logging::log_ssa_inputs(&ssa_args);

        Ok(Self { program, abi, input_map, ssa_args, options, ssa1, ssa2 })
    }

    pub fn exec(&self) -> eyre::Result<CompareInterpretedResult> {
        // Interpret an SSA with a fresh copy of the input values.
        let interpret = |ssa: &Ssa| {
            let mut output = Vec::new();
            let res = ssa.interpret_with_options(
                Value::snapshot_args(&self.ssa_args),
                InterpreterOptions::default(),
                &mut output,
            );
            (res, output)
        };

        Ok(CompareInterpretedResult::new(interpret(&self.ssa1.ssa), interpret(&self.ssa2.ssa)))
    }
}

impl CompareInterpretedResult {
    pub fn new(
        (res1, print1): (InterpretResult, Vec<u8>),
        (res2, print2): (InterpretResult, Vec<u8>),
    ) -> Self {
        let printed = |p| String::from_utf8(p).expect("from_utf8 of print");
        let failed = |e, p| FailedOutput { error: e, print_output: printed(p) };
        let passed = |ret, p| PassedOutput { return_value: Some(ret), print_output: printed(p) };

        match (res1, res2) {
            (Ok(r1), Ok(r2)) => Self::BothPassed(passed(r1, print1), passed(r2, print2)),
            (Ok(r1), Err(e2)) => Self::RightFailed(passed(r1, print1), failed(e2, print2)),
            (Err(e1), Ok(r2)) => Self::LeftFailed(failed(e1, print1), passed(r2, print2)),
            (Err(e1), Err(e2)) => Self::BothFailed(failed(e1, print1), failed(e2, print2)),
        }
    }
}

impl Comparable for ssa::interpreter::errors::InterpreterError {
    fn equivalent(e1: &Self, e2: &Self) -> bool {
        use ssa::interpreter::errors::InternalError;
        use ssa::interpreter::errors::InterpreterError::*;

        match (e1, e2) {
            (
                Internal(InternalError::ConstantDoesNotFitInType { constant: c1, typ: t1 }),
                Internal(InternalError::ConstantDoesNotFitInType { constant: c2, typ: t2 }),
            ) => {
                // The interpreter represents values in types where the result of some casts cannot be represented, while the ACIR and
                // Brillig runtime can fit them into Fields, and defer validation later. We could promote this error to a non-internal one,
                // but the fact remains that the interpreter would fail earlier than ACIR or Brillig.
                // To deal with this we ignore these errors as long as both passes fail the same way.
                c1 == c2 && t1 == t2
            }
            (
                Internal(InternalError::ConstantDoesNotFitInType { constant, .. }),
                RangeCheckFailed { value, .. },
            )
            | (
                RangeCheckFailed { value, .. },
                Internal(InternalError::ConstantDoesNotFitInType { constant, .. }),
            ) => {
                // The value should be a `NumericValue` display format, which is `<type> <value>`.
                let value = value.split_once(' ').map(|(_, value)| value).unwrap_or(value);
                value == constant.to_string()
            }
            (Internal(_), _) | (_, Internal(_)) => {
                // We should not get, or ignore, internal errors.
                // They mean the interpreter got something unexpected that we need to fix.
                false
            }
            (Overflow { instruction: i1, .. }, Overflow { instruction: i2, .. }) => {
                // Overflows can occur or instructions with different IDs, but in a parentheses it contains the values that caused it.
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

            // We expand checked operations on signed types into multiple instructions during the `expand_signed_checks`
            // pass. This results in the error changing from an `Overflow` into a different error type so we match
            // on the attached error message.
            (Overflow { operator, .. }, ConstrainEqFailed { msg: Some(msg), .. }) => match operator
            {
                BinaryOp::Add { unchecked: false } => msg == "attempt to add with overflow",
                BinaryOp::Sub { unchecked: false } => msg == "attempt to subtract with overflow",
                BinaryOp::Mul { unchecked: false } => msg == "attempt to multiply with overflow",
                BinaryOp::Shl | BinaryOp::Shr => {
                    msg == "attempt to bit-shift with overflow"
                        || msg == "attempt to shift right with overflow"
                        || msg == "attempt to shift left with overflow"
                }
                _ => false,
            },
            (
                Overflow { operator: BinaryOp::Mul { unchecked: false }, .. },
                RangeCheckFailed { msg: Some(msg), .. },
            ) => msg == "attempt to multiply with overflow",

            (
                ConstrainEqFailed { msg: msg1, .. } | ConstrainNeFailed { msg: msg1, .. },
                ConstrainEqFailed { msg: msg2, .. } | ConstrainNeFailed { msg: msg2, .. },
            ) => {
                // The `lhs` and `rhs` might change during passes, making direct comparison difficult:
                // * the sides might be flipped: `u1 0 == u1 1` vs `u1 1 == u1 0`
                // * the condition might be flipped: `u1 1 != u1 0` vs `Field 0 == Field 0`
                // * types could change:
                //      * `Field 313339671284855045676773137498590239475 != Field 0` vs `u128 313339671284855045676773137498590239475 != u128 0`
                //      * `i64 -1615928006 != i64 -5568658583620095790` vs `u64 18446744072093623610 != u64 12878085490089455826`
                // So instead of reasoning about the `lhs` and `rhs` formats, let's just compare the message so we know it's the same constraint:
                msg1 == msg2
            }
            (RangeCheckFailed { msg: Some(msg1), .. }, ConstrainEqFailed { msg: msg2, .. }) => {
                // The removal of unreachable instructions evaluates constant binary operations and can replace
                // e.g. a `mul` followed by a `range_check` with a `constrain true == false, "attempt to multiple with overflow"`
                msg2.as_ref().is_some_and(|msg| msg == msg1)
            }
            (DivisionByZero { .. }, ConstrainEqFailed { msg, .. }) => {
                msg.as_ref().is_some_and(|msg| {
                    msg == "attempt to divide by zero" || msg.contains("divisor of zero")
                })
            }
            (DivisionByZero { .. }, DivisionByZero { .. }) => {
                // Signed math in ACIR is expanded to unsigned math. We may have two different `DivisionByZero` errors due to differing types.
                true
            }
            (PoppedFromEmptyList { .. }, ConstrainEqFailed { msg, .. }) => {
                // The removal of unreachable instructions can replace popping from an empty list with an always-fail constraint.
                msg.as_ref().is_some_and(|msg| msg == "Index out of bounds")
            }
            (IndexOutOfBounds { .. }, ConstrainEqFailed { msg, .. }) => {
                msg.as_ref().is_some_and(|msg| msg.contains("Index out of bounds"))
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

impl Comparable for Value {
    fn equivalent(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (Value::ArrayOrList(a), Value::ArrayOrList(b)) => {
                // Ignore the RC
                a.element_types == b.element_types
                    && Comparable::equivalent(&a.elements, &b.elements)
                    && a.is_list == b.is_list
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
pub fn input_values_to_ssa(abi: &Abi, input_map: &InputMap) -> Vec<Value> {
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
/// Tuple types and structs are flattened.
pub fn input_value_to_ssa(typ: &AbiType, input: &InputValue) -> Vec<Value> {
    let mut values = Vec::new();
    append_input_value_to_ssa(typ, input, &mut values);
    values
}

fn append_input_value_to_ssa(typ: &AbiType, input: &InputValue, values: &mut Vec<Value>) {
    use ssa::interpreter::value::{ArrayValue, NumericValue, Value};
    use ssa::ir::types::Type;
    let array_value = |elements: Vec<Value>, types: Vec<Type>| {
        Value::ArrayOrList(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: Arc::new(types),
            is_list: false,
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
            let num_val = NumericValue::from_constant(*f, num_typ).expect("cannot create constant");
            values.push(Value::Numeric(num_val));
        }
        InputValue::String(s) => values
            .push(array_value(vecmap(s.as_bytes(), |b| Value::u8(*b)), vec![Type::unsigned(8)])),
        InputValue::Vec(input_values) => match typ {
            AbiType::Array { length, typ } => {
                assert_eq!(*length as usize, input_values.len(), "array length != input length");
                let mut elements = Vec::with_capacity(*length as usize);
                for input in input_values {
                    append_input_value_to_ssa(typ, input, &mut elements);
                }
                values.push(array_value(elements, input_type_to_ssa(typ)));
            }
            AbiType::Tuple { fields } => {
                assert_eq!(fields.len(), input_values.len(), "tuple size != input length");

                // Tuples are flattened
                for (typ, input) in fields.iter().zip(input_values) {
                    append_input_value_to_ssa(typ, input, values);
                }
            }
            other => {
                panic!("unexpected ABI type for vector input: {other:?}")
            }
        },
        InputValue::Struct(field_values) => match typ {
            AbiType::Struct { path: _, fields } => {
                assert_eq!(fields.len(), field_values.len(), "struct size != input length");

                // Structs are flattened
                for (name, typ) in fields {
                    let input = &field_values[name];
                    append_input_value_to_ssa(typ, input, values);
                }
            }
            other => {
                panic!("unexpected ABI type for map input: {other:?}")
            }
        },
    }
}

/// Convert an ABI type into SSA.
fn input_type_to_ssa(typ: &AbiType) -> Vec<ssa::ir::types::Type> {
    let mut types = Vec::new();
    append_input_type_to_ssa(typ, &mut types);
    types
}

fn append_input_type_to_ssa(typ: &AbiType, types: &mut Vec<ssa::ir::types::Type>) {
    use ssa::ir::types::Type;
    match typ {
        AbiType::Field => types.push(Type::field()),
        AbiType::Array { length, typ } => {
            types.push(Type::Array(Arc::new(input_type_to_ssa(typ)), *length));
        }
        AbiType::Integer { sign: Sign::Signed, width } => types.push(Type::signed(*width)),
        AbiType::Integer { sign: Sign::Unsigned, width } => types.push(Type::unsigned(*width)),
        AbiType::Boolean => types.push(Type::bool()),
        AbiType::Struct { path: _, fields } => {
            // Structs are flattened
            for (_, typ) in fields {
                append_input_type_to_ssa(typ, types);
            }
        }
        AbiType::Tuple { fields } => {
            // Tuples are flattened
            for typ in fields {
                append_input_type_to_ssa(typ, types);
            }
        }
        AbiType::String { length } => {
            types.push(Type::Array(Arc::new(vec![Type::unsigned(8)]), *length));
        }
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
