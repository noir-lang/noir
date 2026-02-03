use acvm::{
    AcirField, FieldElement,
    acir::{
        brillig::{BitSize, HeapVector, IntegerBitSize, MemoryAddress, Opcode as BrilligOpcode},
        circuit::Program,
        native_types::{Witness, WitnessMap},
    },
    assert_circuit_snapshot,
    blackbox_solver::StubbedBlackBoxSolver,
    pwg::{ACVM, ACVMStatus},
};
use noirc_artifacts::debug::DebugInfo;
use noirc_frontend::shared::Visibility;
use std::collections::BTreeMap;

use crate::{
    acir::{acir_context::BrilligStdLib, ssa::codegen_acir},
    brillig::{Brillig, BrilligOptions, brillig_ir::artifact::GeneratedBrillig},
    errors::RuntimeError,
    ssa::{
        ArtifactsAndWarnings, combine_artifacts, interpreter::value::Value, ir::types::NumericType,
        ssa_gen::Ssa,
    },
};
use proptest::prelude::*;

mod arrays;
mod brillig_call;
mod call;
mod instructions;
mod intrinsics;

/// Test utility for converting [ACIR gen artifacts][crate::acir::ssa::Artifacts]
/// into the final [ACIR Program][Program] in order to use its parser and human-readable text format.
fn ssa_to_acir_program(src: &str) -> Program<FieldElement> {
    ssa_to_acir_program_with_debug_info(src).0
}

fn ssa_to_acir_program_with_debug_info(src: &str) -> (Program<FieldElement>, Vec<DebugInfo>) {
    try_ssa_to_acir(src).expect("Should compile manually written SSA into ACIR")
}

/// Attempts to convert SSA to ACIR, returning the error if compilation fails.
fn try_ssa_to_acir(src: &str) -> Result<(Program<FieldElement>, Vec<DebugInfo>), RuntimeError> {
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
                .map(|param| function.dfg.type_of_value(*param).flattened_size().0)
                .sum();
            vec![(param_size, Visibility::Private)]
        })
        .collect::<Vec<_>>();

    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, brillig_functions, _) =
        ssa.generate_entry_point_index().into_acir(&brillig, &BrilligOptions::default())?;

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
    Ok((program, debug))
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
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w0, bits: 32
    BLACKBOX::RANGE input: w1, bits: 32
    ASSERT w2 = w0*w1
    ");
}

#[test]
fn no_zero_bits_range_check() {
    let src = "
    acir(inline) fn main f0 {   
        b0(v0: Field):
            v1 = truncate v0 to 8 bits, max_bit_size: 254
            v2 = cast v1 as u8
            return v2
        }
    ";
    let program = ssa_to_acir_program(src);

    // Check that there is no 0-bits range check, but 'ASSERT w7 = 0' instead
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: [w1]
    BRILLIG CALL func: 0, predicate: 1, inputs: [w0, 256], outputs: [w2, w3]
    BLACKBOX::RANGE input: w2, bits: 246
    BLACKBOX::RANGE input: w3, bits: 8
    ASSERT w3 = w0 - 256*w2
    ASSERT w4 = -w2 + 85500948718122168836900022442411230814642048439125134155071110103811751936
    BLACKBOX::RANGE input: w4, bits: 246
    BRILLIG CALL func: 1, predicate: 1, inputs: [-w2 + 85500948718122168836900022442411230814642048439125134155071110103811751936], outputs: [w5]
    ASSERT w6 = w2*w5 - 85500948718122168836900022442411230814642048439125134155071110103811751936*w5 + 1
    ASSERT 0 = -w2*w6 + 85500948718122168836900022442411230814642048439125134155071110103811751936*w6
    ASSERT 0 = w3*w6
    ASSERT w1 = w3

    unconstrained func 0: directive_integer_quotient
    0: @10 = const u32 2
    1: @11 = const u32 0
    2: @0 = calldata copy [@11; @10]
    3: @2 = field int_div @0, @1
    4: @1 = field mul @2, @1
    5: @1 = field sub @0, @1
    6: @0 = @2
    7: stop @[@11; @10]
    unconstrained func 1: directive_invert
    0: @21 = const u32 1
    1: @20 = const u32 0
    2: @0 = calldata copy [@20; @21]
    3: @2 = const field 0
    4: @3 = field eq @0, @2
    5: jump if @3 to 8
    6: @1 = const field 1
    7: @0 = field field_div @1, @0
    8: stop @[@20; @21]
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
    )
    .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    // [`malicious_directive_quotient`, `directive_invert`]
    assert_eq!(brillig_functions.len(), 2);

    let main = &acir_functions[0];

    let initial_witness = WitnessMap::from(BTreeMap::from([(Witness(0), input)]));
    let blackbox_solver = StubbedBlackBoxSolver;
    let mut acvm =
        ACVM::new(&blackbox_solver, main.opcodes(), initial_witness, &brillig_functions, &[]);

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
        .into_acir(&brillig, &BrilligOptions::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    assert!(acir_functions[0].opcodes().is_empty());
}

#[test]
fn properly_constrains_quotient_when_truncating_fields_to_u128() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = truncate v0 to 128 bits, max_bit_size: 254
            return v1
        }";
    let ssa = Ssa::from_str(src).unwrap();

    let input = FieldElement::zero();
    let malicious_q = FieldElement::try_from_str("64323764613183177041862057485226039389").unwrap();
    let malicious_r = FieldElement::try_from_str("53438638232309528389504892708671455233").unwrap();

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
    )
    .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    // [`malicious_directive_quotient`, `directive_invert`]
    assert_eq!(brillig_functions.len(), 2);

    let main = &acir_functions[0];

    let initial_witness = WitnessMap::from(BTreeMap::from([(Witness(0), input)]));
    let blackbox_solver = StubbedBlackBoxSolver;
    let mut acvm =
        ACVM::new(&blackbox_solver, main.opcodes(), initial_witness, &brillig_functions, &[]);

    assert!(matches!(acvm.solve(), ACVMStatus::Failure::<FieldElement>(_)));
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
    ssa.into_acir(&brillig, &BrilligOptions::default())
        .expect_err("Should fail with assert constant");
}

#[test]
fn databus() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
        b0(v0: u32, v1: u32):
            v2 = cast v0 as Field
            v3 = make_array [v2] : [Field; 1]
            constrain v0 == u32 0
            v4 = add v0, v1
            return v4
        }
    ";
    let program = ssa_to_acir_program(src);

    // Check that w0 is not replaced
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE input: w1, bits: 32
    ASSERT w0 = 0
    ASSERT w3 = w0 + w1
    BLACKBOX::RANGE input: w3, bits: 32
    ASSERT w2 = w3
    ");
}

#[test]
fn databus_deduplicate_call_and_return_data() {
    // call_data and return_data are the same
    let src = "
    acir(inline) pure fn main f0 {
    call_data(0): array: v1, indices: []
    return_data: v1
    b0(v0: Field):
        v1 = make_array [v0] : [Field; 1]
        return v1
    }
    ";
    let program = ssa_to_acir_program(src);

    // Check that RETURNDATA and CALLDATA are distinct blocks
    assert_circuit_snapshot!(program, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    ASSERT w1 = w0
    INIT RETURNDATA b0 = [w1]
    INIT CALLDATA 0 b1 = [w0]
    ");
}

#[test]
fn blake3_slice_regression() {
    // Sanity check for blake3 black box call brillig codegen.
    let src = "
    brillig(inline) predicate_pure fn main f0 {
      b0(v0: [u8; 1]):
        v3 = call blake3(v0) -> [u8; 32]
        return
    }
    ";

    let ssa = Ssa::from_str(src).unwrap();
    execute_ssa(
        ssa,
        WitnessMap::from(BTreeMap::from([(Witness(0), FieldElement::from(104u128))])),
        None,
    );
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
        .into_acir(&brillig, &BrilligOptions::default())
        .expect("Should compile manually written SSA into ACIR");
    assert_eq!(acir_functions.len(), 1);
    let main = &acir_functions[0];
    let blackbox_solver = StubbedBlackBoxSolver;
    let mut acvm =
        ACVM::new(&blackbox_solver, main.opcodes(), initial_witness, &brillig_functions, &[]);
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
            // Both executions failed, so it is the same behavior, as expected.
            (Err(_), (ACVMStatus::Failure(_), _)) => (),
            // Both executions succeeded and output the same value
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
    #[should_panic(expected = "Cannot use `and` with field elements")]
    fn test_and_on_field(lhs in 0u128.., rhs in 0u128..) {
        let lhs = FieldElement::from(lhs);
        let rhs = FieldElement::from(rhs);
        let operators = [
            "and",
        ];
        let inputs = [lhs, rhs];
        test_operators(&operators, "Field", &inputs);
    }

    #[test]
    #[should_panic(expected = "Cannot use `xor` with field elements")]
    fn test_xor_on_field(lhs in 0u128.., rhs in 0u128..) {
        let lhs = FieldElement::from(lhs);
        let rhs = FieldElement::from(rhs);
        let operators = [
            "xor",
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
