use acvm::{
    AcirField, FieldElement,
    acir::{
        brillig::{BitSize, HeapVector, IntegerBitSize, MemoryAddress, Opcode as BrilligOpcode},
        circuit::{ExpressionWidth, Program},
        native_types::{Witness, WitnessMap},
    },
    assert_circuit_snapshot,
    blackbox_solver::StubbedBlackBoxSolver,
    pwg::{ACVM, ACVMStatus},
};
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::shared::Visibility;
use std::collections::BTreeMap;

use crate::{
    acir::{acir_context::BrilligStdLib, ssa::codegen_acir},
    brillig::{Brillig, BrilligOptions, brillig_ir::artifact::GeneratedBrillig},
    ssa::{
        ArtifactsAndWarnings, combine_artifacts, interpreter::value::Value, ir::types::NumericType,
        ssa_gen::Ssa,
    },
};
use proptest::prelude::*;

mod brillig_call;
mod call;
mod intrinsics;

/// Test utility for converting [ACIR gen artifacts][crate::acir::ssa::Artifacts]
/// into the final [ACIR Program][Program] in order to use its parser and human-readable text format.
fn ssa_to_acir_program(src: &str) -> Program<FieldElement> {
    ssa_to_acir_program_with_debug_info(src).0
}

fn ssa_to_acir_program_with_debug_info(src: &str) -> (Program<FieldElement>, Vec<DebugInfo>) {
    let ssa = Ssa::from_str(src).unwrap();
    let arg_size_and_visibilities = ssa
        .functions
        .iter()
        .filter(|(id, function)| {
            function.runtime().is_acir()
                && (**id == ssa.main_id || function.runtime().is_entry_point())
        })
        .map(|(_, function)| {
            // Make all arguments private for the sake of simplicity.
            let param_size: u32 = function
                .parameters()
                .iter()
                .map(|param| function.dfg.type_of_value(*param).flattened_size())
                .sum();
            vec![(param_size, Visibility::Private)]
        })
        .collect::<Vec<_>>();

    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, brillig_functions, _) = ssa
        .generate_entry_point_index()
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    let artifacts =
        ArtifactsAndWarnings((acir_functions, brillig_functions, BTreeMap::default()), vec![]);
    let program_artifact = combine_artifacts(
        artifacts,
        &arg_size_and_visibilities,
        BTreeMap::default(),
        BTreeMap::default(),
        BTreeMap::default(),
    );
    let program = program_artifact.program;
    let debug = program_artifact.debug;
    (program, debug)
}

#[test]
fn unchecked_mul_should_not_have_range_check() {
    let src = "
    acir(inline) fn main f0 {
        b0(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            return v3
        }
    ";
    let program = ssa_to_acir_program(src);

    // Check that range checks only exist on the function parameters
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w2
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    EXPR [ (-1, w0, w1) (1, w2) 0 ]
    ");
}

#[test]
fn does_not_generate_memory_blocks_without_dynamic_accesses() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
            call f1(u32 2, v3)
            v7 = array_get v0, index u32 0 -> Field
            constrain v7 == Field 0
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: [Field]):
              return
          }
        ";
    let program = ssa_to_acir_program(src);
    println!("{program}");

    // Check that no memory opcodes were emitted.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w1
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ 2 ], [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]]], outputs: []
    EXPR [ (1, w0) 0 ]

    unconstrained func 0
    [Const { destination: Direct(2), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Direct(0), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(32836), size_address: Relative(3), offset_address: Relative(4) }, Cast { destination: Direct(32836), source: Direct(32836), bit_size: Integer(U32) }, Mov { destination: Relative(1), source: Direct(32836) }, Const { destination: Relative(2), bit_size: Integer(U32), value: 32837 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 2 }, Const { destination: Relative(6), bit_size: Integer(U32), value: 3 }, BinaryIntOp { destination: Relative(5), op: Add, bit_size: U32, lhs: Relative(4), rhs: Relative(6) }, Mov { destination: Relative(3), source: Direct(1) }, BinaryIntOp { destination: Direct(1), op: Add, bit_size: U32, lhs: Direct(1), rhs: Relative(5) }, IndirectConst { destination_pointer: Relative(3), bit_size: Integer(U32), value: 1 }, BinaryIntOp { destination: Relative(5), op: Add, bit_size: U32, lhs: Relative(3), rhs: Direct(2) }, Store { destination_pointer: Relative(5), source: Relative(4) }, BinaryIntOp { destination: Relative(5), op: Add, bit_size: U32, lhs: Relative(5), rhs: Direct(2) }, Store { destination_pointer: Relative(5), source: Relative(4) }, Const { destination: Relative(6), bit_size: Integer(U32), value: 3 }, BinaryIntOp { destination: Relative(5), op: Add, bit_size: U32, lhs: Relative(3), rhs: Relative(6) }, Mov { destination: Direct(32771), source: Relative(2) }, Mov { destination: Direct(32772), source: Relative(5) }, Mov { destination: Direct(32773), source: Relative(4) }, Call { location: 31 }, Mov { destination: Relative(2), source: Relative(3) }, Call { location: 42 }, Call { location: 43 }, Const { destination: Relative(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Relative(2), bit_size: Integer(U32), value: 0 }, Stop { return_data: HeapVector { pointer: Relative(1), size: Relative(2) } }, BinaryIntOp { destination: Direct(32775), op: Add, bit_size: U32, lhs: Direct(32771), rhs: Direct(32773) }, Mov { destination: Direct(32776), source: Direct(32771) }, Mov { destination: Direct(32777), source: Direct(32772) }, BinaryIntOp { destination: Direct(32778), op: Equals, bit_size: U32, lhs: Direct(32776), rhs: Direct(32775) }, JumpIf { condition: Direct(32778), location: 41 }, Load { destination: Direct(32774), source_pointer: Direct(32776) }, Store { destination_pointer: Direct(32777), source: Direct(32774) }, BinaryIntOp { destination: Direct(32776), op: Add, bit_size: U32, lhs: Direct(32776), rhs: Direct(2) }, BinaryIntOp { destination: Direct(32777), op: Add, bit_size: U32, lhs: Direct(32777), rhs: Direct(2) }, Jump { location: 34 }, Return, Return, Call { location: 45 }, Return, Const { destination: Direct(32772), bit_size: Integer(U32), value: 30720 }, BinaryIntOp { destination: Direct(32771), op: LessThan, bit_size: U32, lhs: Direct(0), rhs: Direct(32772) }, JumpIf { condition: Direct(32771), location: 50 }, IndirectConst { destination_pointer: Direct(1), bit_size: Integer(U64), value: 15764276373176857197 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Direct(2) } }, Return]
    ");
}

#[test]
fn properly_constrains_quotient_when_truncating_fields() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            return v1
        }";
    let ssa = Ssa::from_str(src).unwrap();

    // Here we're attempting to perform a truncation of a `Field` type into 32 bits. We then do a euclidean
    // division `a/b` with `a` and `b` taking the values:
    //
    // a = 0xf9bb18d1ece5fd647afba497e7ea7a2d7cc17b786468f6ebc1e0a6b0fffffff
    // b = 0x100000000 (2**32)
    //
    // We expect q and r to be constrained such that the expression `a = q*b + r` has the single solution.
    //
    // q = 0xf9bb18d1ece5fd647afba497e7ea7a2d7cc17b786468f6ebc1e0a6b
    // r = 0xfffffff
    //
    // One necessary constraint is that q <= field_modulus / b as otherwise `q*b` will overflow the field modulus.
    // Relaxing this constraint permits another solution:
    //
    // malicious_q = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffff
    // malicious_r = 0
    //
    // We then require that if this solution is injected that execution will fail.

    let input =
        FieldElement::from_hex("0xf9bb18d1ece5fd647afba497e7ea7a2d7cc17b786468f6ebc1e0a6b0fffffff")
            .unwrap();
    let malicious_q =
        FieldElement::from_hex("0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffff")
            .unwrap();
    let malicious_r = FieldElement::zero();

    // This brillig function replaces the standard implementation of `directive_quotient` with
    // an implementation which returns `(malicious_q, malicious_r)`.
    let malicious_quotient = GeneratedBrillig {
        byte_code: vec![
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(10),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(2_usize),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(11),
                bit_size: BitSize::Integer(IntegerBitSize::U32),
                value: FieldElement::from(0_usize),
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(0),
                bit_size: BitSize::Field,
                value: malicious_q,
            },
            BrilligOpcode::Const {
                destination: MemoryAddress::direct(1),
                bit_size: BitSize::Field,
                value: malicious_r,
            },
            BrilligOpcode::Stop {
                return_data: HeapVector {
                    pointer: MemoryAddress::direct(11),
                    size: MemoryAddress::direct(10),
                },
            },
        ],
        name: "malicious_directive_quotient".to_string(),
        ..Default::default()
    };

    let malicious_brillig_stdlib =
        BrilligStdLib { quotient: malicious_quotient, ..BrilligStdLib::default() };

    let (acir_functions, brillig_functions, _) = codegen_acir(
        ssa,
        &Brillig::default(),
        malicious_brillig_stdlib,
        &BrilligOptions::default(),
        ExpressionWidth::default(),
    )
    .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    // [`malicious_directive_quotient`, `directive_invert`]
    assert_eq!(brillig_functions.len(), 2);

    let main = &acir_functions[0];

    let initial_witness = WitnessMap::from(BTreeMap::from([(Witness(0), input)]));
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver(true),
        main.opcodes(),
        initial_witness,
        &brillig_functions,
        &[],
    );

    assert!(matches!(acvm.solve(), ACVMStatus::Failure::<FieldElement>(_)));
}

#[test]
fn do_not_overflow_with_constant_constrain_neq() {
    // Test that we appropriately fetch the assertion payload opcode location.
    // We expect this constrain neq to be simplified and not lay down any opcodes.
    // As the constrain neq is the first opcode, if we do not fetch the last opcode
    // location correctly we can potentially trigger an overflow.
    let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain Field 1 != Field 0, ""
            return
        }
        "#;
    let ssa = Ssa::from_str(src).unwrap();
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, _brillig_functions, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    assert!(acir_functions[0].opcodes().is_empty());
}

#[test]
fn derive_pedersen_generators_requires_constant_input() {
    // derive_pedersen_generators is expected to fail because one of its argument is not a constant.
    let src = r#"
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32):
        separator = make_array b"DEFAULT_DOMAIN_SEPARATOR"
        v2 = call derive_pedersen_generators(separator, v1) -> [(Field, Field, u1); 1]
        return v2
    }
    "#;

    let ssa = Ssa::from_str(src).unwrap();
    let brillig = ssa.to_brillig(&BrilligOptions::default());
    ssa.into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect_err("Should fail with assert constant");
}

#[test]
// Regression for https://github.com/noir-lang/noir/issues/9847
fn signed_div_overflow() {
    // Test that check -128 / -1 overflow for i8
    let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v1: i8, v2: i8):
            v3 = div v1, v2
            return
        }
        "#;

    let ssa = Ssa::from_str(src).unwrap();
    let inputs = vec![FieldElement::from(128_u128), FieldElement::from(255_u128)];
    let inputs = inputs
        .into_iter()
        .enumerate()
        .map(|(i, f)| (Witness(i as u32), f))
        .collect::<BTreeMap<_, _>>();
    let initial_witness = WitnessMap::from(inputs);
    let output = None;

    // acir execution should fail to divide -128 / -1
    let acir_execution_result = execute_ssa(ssa, initial_witness.clone(), output.as_ref());
    assert!(matches!(acir_execution_result, (ACVMStatus::Failure(_), _)));
}

/// Convert the SSA input into ACIR and use ACVM to execute it
/// Returns the ACVM execution status and the value of the 'output' witness value,
/// unless the provided output is None or the ACVM fails during execution.
fn execute_ssa(
    ssa: Ssa,
    initial_witness: WitnessMap<FieldElement>,
    output: Option<&Witness>,
) -> (ACVMStatus<FieldElement>, Option<FieldElement>) {
    let brillig = ssa.to_brillig(&BrilligOptions::default());
    let (acir_functions, brillig_functions, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");
    assert_eq!(acir_functions.len(), 1);
    let main = &acir_functions[0];
    let mut acvm = ACVM::new(
        &StubbedBlackBoxSolver(true),
        main.opcodes(),
        initial_witness,
        &brillig_functions,
        &[],
    );
    let status = acvm.solve();
    if status == ACVMStatus::Solved {
        (status, output.map(|o| acvm.witness_map()[o]))
    } else {
        (status, None)
    }
}

fn get_main_src(typ: &str) -> String {
    format!(
        "acir(inline) fn main f0 {{
            b0(inputs: [{typ}; 2]):
              lhs = array_get inputs, index u32 0 -> {typ}
              rhs = array_get inputs, index u32 1 -> {typ}
              "
    )
}

/// Create a SSA instruction corresponding to the operator, using v1 and v2 as operands.
/// Additional information can be added to the string,
/// for instance, "range_check 8" creates 'range_check v1 to 8 bits'
fn generate_test_instruction_from_operator(operator: &str) -> (String, bool) {
    let ops = operator.split(" ").collect::<Vec<_>>();
    let op = ops[0];
    let mut output = true;
    let src = match op {
        "constrain" => {
            output = false;
            format!("constrain lhs {} rhs", ops[1])
        }
        "not" => format!("result = {op} lhs"),
        "truncate" => {
            format!("result = truncate lhs to {} bits, max_bit_size: {}", ops[1], ops[2])
        }
        "range_check" => {
            output = false;
            format!("range_check lhs to {} bits", ops[1])
        }
        _ => format!("result = {op} lhs, rhs"),
    };

    if output {
        (
            format!(
                "
            {src}
        return result
        }}"
            ),
            true,
        )
    } else {
        (
            format!(
                "
            {src}
        return
        }}"
            ),
            false,
        )
    }
}

/// Execute a simple operation for each operators
/// The operation is executed from SSA IR using ACVM after acir-gen
/// and also directly on the SSA IR using the SSA interpreter.
/// The results are compared to ensure that both executions yield the same result.
fn test_operators(
    // The list of operators to test
    operators: &[&str],
    // the type of the input values
    typ: &str,
    // the values of the inputs
    inputs: &[FieldElement],
) {
    let main = get_main_src(typ);
    let num_type = match typ.chars().next().unwrap() {
        'F' => NumericType::NativeField,
        'i' => NumericType::Signed { bit_size: typ[1..].parse().unwrap() },
        'u' => NumericType::Unsigned { bit_size: typ[1..].parse().unwrap() },
        _ => unreachable!("invalid numeric type"),
    };
    let inputs_int = Value::array_from_iter(inputs.iter().cloned(), num_type).unwrap();
    let inputs =
        inputs.iter().enumerate().map(|(i, f)| (Witness(i as u32), *f)).collect::<BTreeMap<_, _>>();
    let len = inputs.len() as u32;
    let initial_witness = WitnessMap::from(inputs);

    for op in operators {
        let (src, with_output) = generate_test_instruction_from_operator(op);
        let output = if with_output { Some(Witness(len)) } else { None };
        let ssa = Ssa::from_str(&(main.to_owned() + &src)).unwrap();
        // ssa execution
        let ssa_interpreter_result = ssa.interpret(vec![inputs_int.clone()]);
        // acir execution
        let acir_execution_result = execute_ssa(ssa, initial_witness.clone(), output.as_ref());

        match (ssa_interpreter_result, acir_execution_result) {
            // Both execution failed, so it is the same behavior, as expected.
            (Err(_), (ACVMStatus::Failure(_), _)) => (),
            // Both execution succeeded and output the same value
            (Ok(ssa_inner_result), (ACVMStatus::Solved, acvm_result)) => {
                let ssa_result = if let Some(result) = ssa_inner_result.first() {
                    result.as_numeric().map(|v| v.convert_to_field())
                } else {
                    None
                };
                assert_eq!(ssa_result, acvm_result);
            }
            _ => panic!("ssa and acvm execution should have the same result"),
        }
    }
}

proptest! {
#[test]
fn test_binary_on_field(lhs in 0u128.., rhs in 0u128..) {
            let lhs = FieldElement::from(lhs);
            let rhs = FieldElement::from(rhs);
    // Test the following Binary operation on Fields
    let operators = [
        "add",
        "sub",
        "mul",
        "div",
        "eq",
        // Bitwise operations are not allowed on field elements
        // SSA interpreter will emit an error but not ACVM
        // "and",
        // "xor",
        "unchecked_add",
        "unchecked_sub",
        "unchecked_mul",
        "range_check 32",
        "truncate 32 254",
    ];
    let inputs = [lhs, rhs];
    test_operators(&operators, "Field", &inputs);
}

#[test]
fn test_u32(lhs in 0u32.., rhs in 0u32..) {
    let lhs = FieldElement::from(lhs);
    let rhs = FieldElement::from(rhs);

    // Test the following operations on u32
    let operators = [
        "add",
        "sub",
        "mul",
        "div",
        "eq",
        "and",
        "xor",
        "mod",
        "lt",
        "or",
        "not",
        "range_check 8",
        "truncate 8 32",
    ];
    let inputs = [lhs, rhs];
    test_operators(&operators, "u32", &inputs);

    //unchecked operations assume no under/over-flow
    let mut unchecked_operators = vec![];
    if (lhs + rhs).to_u128() <= u128::from(u32::MAX) {
        unchecked_operators.push("unchecked_add");
    }
    if (lhs * rhs).to_u128() <= u128::from(u32::MAX) {
    unchecked_operators.push("unchecked_mul");
    }
    if lhs >= rhs {
        unchecked_operators.push("unchecked_sub");
    }
    test_operators(&unchecked_operators, "u32", &inputs);
}


 #[test]
fn test_constraint_field(lhs in 0u128.., rhs in 0u128..) {
    let lhs = FieldElement::from(lhs);
    let rhs = FieldElement::from(rhs);
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "Field", &[lhs,rhs]);
    test_operators(&operators, "u128", &[lhs,rhs]);
}

#[test]
fn test_constraint_u32(lhs in 0u32.., rhs in 0u32..) {
    let lhs = FieldElement::from(lhs);
    let rhs = FieldElement::from(rhs);
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u32", &[lhs,rhs]);
    test_operators(&operators, "i32", &[lhs,rhs]);
}

#[test]
fn test_constraint_u64(lhs in 0u64.., rhs in 0u64..) {
    let lhs = FieldElement::from(lhs);
    let rhs = FieldElement::from(rhs);
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u64", &[lhs,rhs]);
    test_operators(&operators, "i64", &[lhs,rhs]);
}

#[test]
fn test_constraint_u16(lhs in 0u16.., rhs in 0u16..) {
    let lhs = FieldElement::from(u128::from(lhs));
    let rhs = FieldElement::from(u128::from(rhs));
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u16", &[lhs,rhs]);
    test_operators(&operators, "i16", &[lhs,rhs]);
}

#[test]
fn test_constraint_u8(lhs in 0u8.., rhs in 0u8..) {
    let lhs = FieldElement::from(u128::from(lhs));
    let rhs = FieldElement::from(u128::from(rhs));
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u8", &[lhs,rhs]);
    test_operators(&operators, "i8", &[lhs,rhs]);
}

}
