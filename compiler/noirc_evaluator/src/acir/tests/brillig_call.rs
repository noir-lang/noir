use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::circuit::{ExpressionWidth, Opcode, OpcodeLocation, brillig::BrilligFunctionId},
    assert_circuit_snapshot,
};
use noirc_frontend::monomorphization::ast::InlineType;

use crate::{
    acir::{
        acir_context::BrilligStdlibFunc,
        tests::{build_basic_foo_with_return, ssa_to_acir_program_with_debug_info},
    },
    brillig::BrilligOptions,
    ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::BinaryOp,
            map::Id,
            types::{NumericType, Type},
        },
    },
};

// Test that given multiple calls to the same brillig function we generate only one bytecode
// and the appropriate Brillig call opcodes are generated
#[test]
fn multiple_brillig_calls_one_bytecode() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v4 = call f1(v0, v1) -> Field
        v5 = call f1(v0, v1) -> Field
        v6 = call f1(v0, v1) -> Field
        v7 = call f2(v0, v1) -> Field
        v8 = call f1(v0, v1) -> Field
        v9 = call f2(v0, v1) -> Field
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    brillig(inline) fn foo f2 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    let main_debug = &debug[0];
    // We have two normal Brillig functions that were called multiple times.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(main_debug.brillig_locations.len(), 2);
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(0)));
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(1)));

    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w7
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w2]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w3]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w5]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w6]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w7]
    ");
}

// Test that given multiple primitive operations that are represented by Brillig directives (e.g. invert/quotient),
// we will only generate one bytecode and the appropriate Brillig call opcodes are generated.
#[test]
fn multiple_brillig_stdlib_calls() {
    let src = "
    acir(inline) fn main f0 {
        b0(v0: u32, v1: u32, v2: u32):
          v3 = div v0, v1
          constrain v3 == v2
          v4 = div v1, v2
          constrain v4 == u32 1
          return
    }";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);
    // We expect two brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    assert_eq!(
        program.unconstrained_functions.len(),
        2,
        "Should only have generated two Brillig functions"
    );
    assert_eq!(
        debug[0].brillig_locations.len(),
        0,
        "Brillig stdlib functions do not have location information"
    );

    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    BLACKBOX::RANGE [(w2, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w1, w3) -1 ]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4, w5]
    BLACKBOX::RANGE [(w4, 32)] []
    BLACKBOX::RANGE [(w5, 32)] []
    EXPR [ (1, w1) (-1, w5) (-1, w6) -1 ]
    BLACKBOX::RANGE [(w6, 32)] []
    EXPR [ (-1, w1, w4) (1, w0) (-1, w5) 0 ]
    EXPR [ (-1, w2) (1, w4) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w2) 0 ]], outputs: [w7]
    EXPR [ (1, w2, w7) -1 ]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w1) 0 ], EXPR [ (1, w2) 0 ]], outputs: [w8, w9]
    BLACKBOX::RANGE [(w9, 32)] []
    EXPR [ (1, w2) (-1, w9) (-1, w10) -1 ]
    BLACKBOX::RANGE [(w10, 32)] []
    EXPR [ (-1, w2, w8) (1, w1) (-1, w9) 0 ]
    EXPR [ (1, w8) -1 ]
    ");
}

// Test that given both hardcoded Brillig directives and calls to normal Brillig functions,
// we generate a single bytecode for the directives and a single bytecode for the normal Brillig calls.
#[test]
fn brillig_stdlib_calls_with_regular_brillig_call() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = div v0, v1
        constrain v4 == v2
        v5 = call f1(v0, v1) -> Field
        v6 = call f1(v0, v1) -> Field
        v7 = div v1, v2
        constrain v7 == u32 1
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(
        program.unconstrained_functions.len(),
        3,
        "Should only have generated three Brillig functions"
    );
    // We have one normal Brillig functions that was called twice.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(debug[0].brillig_locations.len(), 1);
    assert!(debug[0].brillig_locations.contains_key(&BrilligFunctionId(0)));

    // Brillig stdlib IDs are expected to always come at the end of the Brillig functions list.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w12
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    BLACKBOX::RANGE [(w2, 32)] []
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w1, w3) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4, w5]
    BLACKBOX::RANGE [(w4, 32)] []
    BLACKBOX::RANGE [(w5, 32)] []
    EXPR [ (1, w1) (-1, w5) (-1, w6) -1 ]
    BLACKBOX::RANGE [(w6, 32)] []
    EXPR [ (-1, w1, w4) (1, w0) (-1, w5) 0 ]
    EXPR [ (-1, w2) (1, w4) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w7]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w8]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w2) 0 ]], outputs: [w9]
    EXPR [ (1, w2, w9) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w1) 0 ], EXPR [ (1, w2) 0 ]], outputs: [w10, w11]
    BLACKBOX::RANGE [(w11, 32)] []
    EXPR [ (1, w2) (-1, w11) (-1, w12) -1 ]
    BLACKBOX::RANGE [(w12, 32)] []
    EXPR [ (-1, w2, w10) (1, w1) (-1, w11) 0 ]
    EXPR [ (1, w10) -1 ]
    ");
}

// Test that given both normal Brillig calls, Brillig stdlib calls, and non-inlined ACIR calls, that we accurately generate ACIR.
#[test]
fn brillig_stdlib_calls_with_multiple_acir_calls() {
    // acir(inline) fn main f0 {
    //     b0(v0: u32, v1: u32, v2: u32):
    //       v4 = div v0, v1
    //       constrain v4 == v2
    //       v5 = call f1(v0, v1)
    //       v6 = call f2(v0, v1)
    //       v7 = div v1, v2
    //       constrain v7 == u32 1
    //       return
    // }
    // brillig fn foo f1 {
    //   b0(v0: Field, v1: Field):
    //     v2 = eq v0, v1
    //     constrain v2 == u1 0
    //     return v0
    // }
    // acir(fold) fn foo f2 {
    //     b0(v0: Field, v1: Field):
    //       v2 = eq v0, v1
    //       constrain v2 == u1 0
    //       return v0
    //   }
    // }
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::unsigned(32));
    let main_v1 = builder.add_parameter(Type::unsigned(32));
    let main_v2 = builder.add_parameter(Type::unsigned(32));

    let foo_id = Id::test_new(1);
    let foo = builder.import_function(foo_id);
    let bar_id = Id::test_new(2);
    let bar = builder.import_function(bar_id);

    // Call a primitive operation that uses Brillig
    let v0_div_v1 = builder.insert_binary(main_v0, BinaryOp::Div, main_v1);
    builder.insert_constrain(v0_div_v1, main_v2, None);

    // Insert multiple calls to the same Brillig function
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();

    // Call the same primitive operation again
    let v1_div_v2 = builder.insert_binary(main_v1, BinaryOp::Div, main_v2);
    let one = builder.numeric_constant(1u128, NumericType::unsigned(32));
    builder.insert_constrain(v1_div_v2, one, None);

    builder.terminate_with_return(vec![]);

    // Build a Brillig function
    build_basic_foo_with_return(&mut builder, foo_id, true, InlineType::default());
    // Build an ACIR function which has the same logic as the Brillig function above
    build_basic_foo_with_return(&mut builder, bar_id, false, InlineType::Fold);

    let ssa = builder.finish();
    // We need to generate  Brillig artifacts for the regular Brillig function and pass them to the ACIR generation pass.
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, brillig_functions, _, _) = ssa
        .generate_entry_point_index()
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 2, "Should only have two ACIR functions");
    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(brillig_functions.len(), 3, "Should only have generated three Brillig functions");

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    check_brillig_calls(&acir_functions[0].brillig_stdlib_func_locations, main_opcodes, 1, 4, 2);

    assert_eq!(main_acir.brillig_locations.len(), 1);
    assert!(main_acir.brillig_locations.contains_key(&BrilligFunctionId(0)));

    let foo_acir = &acir_functions[1];
    let foo_opcodes = foo_acir.opcodes();
    check_brillig_calls(&acir_functions[1].brillig_stdlib_func_locations, foo_opcodes, 1, 1, 0);

    assert_eq!(foo_acir.brillig_locations.len(), 0);
}

fn check_brillig_calls(
    brillig_stdlib_function_locations: &BTreeMap<OpcodeLocation, BrilligStdlibFunc>,
    opcodes: &[Opcode<FieldElement>],
    num_normal_brillig_functions: u32,
    expected_num_stdlib_calls: u32,
    expected_num_normal_calls: u32,
) {
    // First we check calls to the Brillig stdlib
    let mut num_brillig_stdlib_calls = 0;
    for (i, (opcode_location, brillig_stdlib_func)) in
        brillig_stdlib_function_locations.iter().enumerate()
    {
        // We can take just modulo 2 to determine the expected ID as we only code generated two Brillig stdlib function
        let stdlib_func_index = (i % 2) as u32;
        if stdlib_func_index == 0 {
            assert!(matches!(brillig_stdlib_func, BrilligStdlibFunc::Inverse));
        } else {
            assert!(matches!(brillig_stdlib_func, BrilligStdlibFunc::Quotient));
        }

        match opcode_location {
            OpcodeLocation::Acir(acir_index) => {
                match opcodes[*acir_index] {
                    Opcode::BrilligCall { id, .. } => {
                        // Brillig stdlib function calls are only resolved at the end of ACIR generation so their
                        // IDs are expected to always reference Brillig bytecode at the end of the Brillig functions list.
                        // We have one normal Brillig call so we add one here to the std lib function's index within the std lib.
                        let expected_id = stdlib_func_index + num_normal_brillig_functions;
                        let expected_id = BrilligFunctionId(expected_id);
                        assert_eq!(id, expected_id, "Expected {expected_id} but got {id}");
                        num_brillig_stdlib_calls += 1;
                    }
                    _ => panic!("Expected BrilligCall opcode"),
                }
            }
            _ => panic!("Expected OpcodeLocation::Acir"),
        }
    }

    assert_eq!(
        num_brillig_stdlib_calls, expected_num_stdlib_calls,
        "Should have {expected_num_stdlib_calls} BrilligCall opcodes to stdlib functions but got {num_brillig_stdlib_calls}"
    );

    // Check the normal Brillig calls
    // This check right now expects to only call one Brillig function.
    let mut num_normal_brillig_calls = 0;
    for (i, opcode) in opcodes.iter().enumerate() {
        if let Opcode::BrilligCall { id, .. } = opcode {
            if brillig_stdlib_function_locations.get(&OpcodeLocation::Acir(i)).is_some() {
                // We should have already checked Brillig stdlib functions and only want to check normal Brillig calls here
                continue;
            }
            // We only generate one normal Brillig call so we should expect a function ID of `0`
            let expected_id = BrilligFunctionId(0);
            assert_eq!(*id, expected_id, "Expected an id of {expected_id} but got {id}");
            num_normal_brillig_calls += 1;
        }
    }

    assert_eq!(
        num_normal_brillig_calls, expected_num_normal_calls,
        "Should have {expected_num_normal_calls} BrilligCall opcodes to normal Brillig functions but got {num_normal_brillig_calls}"
    );
}
