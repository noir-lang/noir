use acvm::{
    AcirField, FieldElement,
    acir::{
        brillig::{BitSize, HeapVector, IntegerBitSize, MemoryAddress, Opcode as BrilligOpcode},
        circuit::{
            ExpressionWidth, Opcode, OpcodeLocation,
            brillig::BrilligFunctionId,
            opcodes::{AcirFunctionId, BlackBoxFuncCall},
        },
        native_types::{Witness, WitnessMap},
    },
    blackbox_solver::StubbedBlackBoxSolver,
    pwg::{ACVM, ACVMStatus},
};
use noirc_errors::Location;
use noirc_frontend::monomorphization::ast::InlineType;
use std::collections::BTreeMap;

use crate::{
    acir::{BrilligStdlibFunc, acir_context::BrilligStdLib, ssa::codegen_acir},
    brillig::{Brillig, BrilligOptions, brillig_ir::artifact::GeneratedBrillig},
    ssa::{
        function_builder::FunctionBuilder,
        interpreter::value::Value,
        ir::{
            function::FunctionId,
            instruction::BinaryOp,
            map::Id,
            types::{NumericType, Type},
        },
        ssa_gen::Ssa,
    },
};
use proptest::prelude::*;

mod intrinsics;

fn build_basic_foo_with_return(
    builder: &mut FunctionBuilder,
    foo_id: FunctionId,
    brillig: bool,
    inline_type: InlineType,
) {
    // fn foo f1 {
    // b0(v0: Field, v1: Field):
    //     v2 = eq v0, v1
    //     constrain v2 == u1 0
    //     return v0
    // }
    if brillig {
        builder.new_brillig_function("foo".into(), foo_id, inline_type);
    } else {
        builder.new_function("foo".into(), foo_id, inline_type);
    }
    // Set a call stack for testing whether `brillig_locations` in the `GeneratedAcir` was accurately set.
    let stack = vec![Location::dummy(), Location::dummy()];
    let call_stack = builder.current_function.dfg.call_stack_data.get_or_insert_locations(&stack);
    builder.set_call_stack(call_stack);

    let foo_v0 = builder.add_parameter(Type::field());
    let foo_v1 = builder.add_parameter(Type::field());

    let foo_equality_check = builder.insert_binary(foo_v0, BinaryOp::Eq, foo_v1);
    let zero = builder.numeric_constant(0u128, NumericType::unsigned(1));
    builder.insert_constrain(foo_equality_check, zero, None);
    builder.terminate_with_return(vec![foo_v0]);
}

/// Check that each `InlineType` which prevents inlining functions generates code in the same manner
#[test]
fn basic_calls_fold() {
    basic_call_with_outputs_assert(InlineType::Fold);
    call_output_as_next_call_input(InlineType::Fold);
    basic_nested_call(InlineType::Fold);
}

#[test]
#[should_panic = "internal error: entered unreachable code: Expected an associated final index for call to acir function f1 with args [Id(0), Id(1)]"]
fn basic_calls_no_predicates() {
    basic_call_with_outputs_assert(InlineType::NoPredicates);
    call_output_as_next_call_input(InlineType::NoPredicates);
    basic_nested_call(InlineType::NoPredicates);
}

#[test]
#[should_panic = "ICE: Got an ACIR function named foo that should have already been inlined"]
fn call_without_inline_attribute() {
    basic_call_with_outputs_assert(InlineType::Inline);
}

fn basic_call_with_outputs_assert(inline_type: InlineType) {
    // acir(inline) fn main f0 {
    //     b0(v0: Field, v1: Field):
    //       v2 = call f1(v0, v1)
    //       v3 = call f1(v0, v1)
    //       constrain v2 == v3
    //       return
    //     }
    // acir(fold) fn foo f1 {
    //     b0(v0: Field, v1: Field):
    //       v2 = eq v0, v1
    //       constrain v2 == u1 0
    //       return v0
    //     }
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::field());
    let main_v1 = builder.add_parameter(Type::field());

    let foo_id = Id::test_new(1);
    let foo = builder.import_function(foo_id);
    let main_call1_results =
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    let main_call2_results =
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
    builder.terminate_with_return(vec![]);

    build_basic_foo_with_return(&mut builder, foo_id, false, inline_type);

    let ssa = builder.finish().generate_entry_point_index();

    let (acir_functions, _, _, _) = ssa
        .into_acir(&Brillig::default(), &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");
    // Expected result:
    // main f0
    // GeneratedAcir {
    //     ...
    //     opcodes: [
    //         CALL func 1: inputs: [Witness(0), Witness(1)], outputs: [Witness(2)],
    //         CALL func 1: inputs: [Witness(0), Witness(1)], outputs: [Witness(3)],
    //         EXPR [ (1, _2) (-1, _3) 0 ],
    //     ],
    //     return_witnesses: [],
    //     input_witnesses: [
    //         Witness(
    //             0,
    //         ),
    //         Witness(
    //             1,
    //         ),
    //     ],
    //     ...
    // }
    // foo f1
    // GeneratedAcir {
    //     ...
    //     opcodes: [
    //         Same as opcodes as the expected result of `basic_call_codegen`
    //     ],
    //     return_witnesses: [
    //         Witness(
    //             0,
    //         ),
    //     ],
    //     input_witnesses: [
    //         Witness(
    //             0,
    //         ),
    //         Witness(
    //             1,
    //         ),
    //     ],
    //     ...
    // },

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo`");

    check_call_opcode(
        &main_opcodes[0],
        AcirFunctionId(1),
        vec![Witness(0), Witness(1)],
        vec![Witness(2)],
    );
    check_call_opcode(
        &main_opcodes[1],
        AcirFunctionId(1),
        vec![Witness(0), Witness(1)],
        vec![Witness(3)],
    );

    if let Opcode::AssertZero(expr) = &main_opcodes[2] {
        assert_eq!(expr.linear_combinations[0].0, FieldElement::from(1u128));
        assert_eq!(expr.linear_combinations[0].1, Witness(2));

        assert_eq!(expr.linear_combinations[1].0, FieldElement::from(-1i128));
        assert_eq!(expr.linear_combinations[1].1, Witness(3));
        assert_eq!(expr.q_c, FieldElement::from(0u128));
    }
}

fn call_output_as_next_call_input(inline_type: InlineType) {
    // acir(inline) fn main f0 {
    //     b0(v0: Field, v1: Field):
    //       v3 = call f1(v0, v1)
    //       v4 = call f1(v3, v1)
    //       constrain v3 == v4
    //       return
    //     }
    // acir(fold) fn foo f1 {
    //     b0(v0: Field, v1: Field):
    //       v2 = eq v0, v1
    //       constrain v2 == u1 0
    //       return v0
    //     }
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::field());
    let main_v1 = builder.add_parameter(Type::field());

    let foo_id = Id::test_new(1);
    let foo = builder.import_function(foo_id);
    let main_call1_results =
        builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    let main_call2_results = builder
        .insert_call(foo, vec![main_call1_results[0], main_v1], vec![Type::field()])
        .to_vec();
    builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
    builder.terminate_with_return(vec![]);

    build_basic_foo_with_return(&mut builder, foo_id, false, inline_type);

    let ssa = builder.finish();

    let (acir_functions, _, _, _) = ssa
        .generate_entry_point_index()
        .into_acir(&Brillig::default(), &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");
    // The expected result should look very similar to the above test expect that the input witnesses of the `Call`
    // opcodes will be different. The changes can discerned from the checks below.

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo` and an assert");

    check_call_opcode(
        &main_opcodes[0],
        AcirFunctionId(1),
        vec![Witness(0), Witness(1)],
        vec![Witness(2)],
    );
    // The output of the first call should be the input of the second call
    check_call_opcode(
        &main_opcodes[1],
        AcirFunctionId(1),
        vec![Witness(2), Witness(1)],
        vec![Witness(3)],
    );
}

fn basic_nested_call(inline_type: InlineType) {
    // SSA for the following Noir program:
    // fn main(x: Field, y: pub Field) {
    //     let z = func_with_nested_foo_call(x, y);
    //     let z2 = func_with_nested_foo_call(x, y);
    //     assert(z == z2);
    // }
    // #[fold]
    // fn func_with_nested_foo_call(x: Field, y: Field) -> Field {
    //     nested_call(x + 2, y)
    // }
    // #[fold]
    // fn foo(x: Field, y: Field) -> Field {
    //     assert(x != y);
    //     x
    // }
    //
    // SSA:
    // acir(inline) fn main f0 {
    //     b0(v0: Field, v1: Field):
    //       v3 = call f1(v0, v1)
    //       v4 = call f1(v0, v1)
    //       constrain v3 == v4
    //       return
    //     }
    // acir(fold) fn func_with_nested_foo_call f1 {
    //     b0(v0: Field, v1: Field):
    //       v3 = add v0, Field 2
    //       v5 = call f2(v3, v1)
    //       return v5
    //   }
    // acir(fold) fn foo f2 {
    //     b0(v0: Field, v1: Field):
    //       v2 = eq v0, v1
    //       constrain v2 == Field 0
    //       return v0
    //   }
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::field());
    let main_v1 = builder.add_parameter(Type::field());

    let func_with_nested_foo_call_id = Id::test_new(1);
    let func_with_nested_foo_call = builder.import_function(func_with_nested_foo_call_id);
    let main_call1_results = builder
        .insert_call(func_with_nested_foo_call, vec![main_v0, main_v1], vec![Type::field()])
        .to_vec();
    let main_call2_results = builder
        .insert_call(func_with_nested_foo_call, vec![main_v0, main_v1], vec![Type::field()])
        .to_vec();
    builder.insert_constrain(main_call1_results[0], main_call2_results[0], None);
    builder.terminate_with_return(vec![]);

    builder.new_function(
        "func_with_nested_foo_call".into(),
        func_with_nested_foo_call_id,
        inline_type,
    );
    let func_with_nested_call_v0 = builder.add_parameter(Type::field());
    let func_with_nested_call_v1 = builder.add_parameter(Type::field());

    let two = builder.field_constant(2u128);
    let v0_plus_two =
        builder.insert_binary(func_with_nested_call_v0, BinaryOp::Add { unchecked: false }, two);

    let foo_id = Id::test_new(2);
    let foo_call = builder.import_function(foo_id);
    let foo_call = builder
        .insert_call(foo_call, vec![v0_plus_two, func_with_nested_call_v1], vec![Type::field()])
        .to_vec();
    builder.terminate_with_return(vec![foo_call[0]]);

    build_basic_foo_with_return(&mut builder, foo_id, false, inline_type);

    let ssa = builder.finish().generate_entry_point_index();

    let (acir_functions, _, _, _) = ssa
        .into_acir(&Brillig::default(), &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 3, "Should have three ACIR functions");

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    assert_eq!(main_opcodes.len(), 3, "Should have two calls to `foo` and an assert");

    // Both of these should call func_with_nested_foo_call f1
    check_call_opcode(
        &main_opcodes[0],
        AcirFunctionId(1),
        vec![Witness(0), Witness(1)],
        vec![Witness(2)],
    );
    // The output of the first call should be the input of the second call
    check_call_opcode(
        &main_opcodes[1],
        AcirFunctionId(1),
        vec![Witness(0), Witness(1)],
        vec![Witness(3)],
    );

    let func_with_nested_call_acir = &acir_functions[1];
    let func_with_nested_call_opcodes = func_with_nested_call_acir.opcodes();

    assert_eq!(
        func_with_nested_call_opcodes.len(),
        3,
        "Should have an expression and a call to a nested `foo`"
    );
    // Should call foo f2
    check_call_opcode(
        &func_with_nested_call_opcodes[1],
        AcirFunctionId(2),
        vec![Witness(3), Witness(1)],
        vec![Witness(4)],
    );
}

fn check_call_opcode(
    opcode: &Opcode<FieldElement>,
    expected_id: AcirFunctionId,
    expected_inputs: Vec<Witness>,
    expected_outputs: Vec<Witness>,
) {
    match opcode {
        Opcode::Call { id, inputs, outputs, .. } => {
            assert_eq!(*id, expected_id, "Main was expected to call {expected_id} but got {}", *id);
            for (expected_input, input) in expected_inputs.iter().zip(inputs) {
                assert_eq!(
                    expected_input, input,
                    "Expected input witness {expected_input:?} but got {input:?}"
                );
            }
            for (expected_output, output) in expected_outputs.iter().zip(outputs) {
                assert_eq!(
                    expected_output, output,
                    "Expected output witness {expected_output:?} but got {output:?}"
                );
            }
        }
        _ => panic!("Expected only Call opcode"),
    }
}

// Test that given multiple calls to the same brillig function we generate only one bytecode
// and the appropriate Brillig call opcodes are generated
#[test]
fn multiple_brillig_calls_one_bytecode() {
    // acir(inline) fn main f0 {
    //     b0(v0: Field, v1: Field):
    //       v4 = call f1(v0, v1)
    //       v5 = call f1(v0, v1)
    //       v6 = call f1(v0, v1)
    //       v7 = call f2(v0, v1)
    //       v8 = call f1(v0, v1)
    //       v9 = call f2(v0, v1)
    //       return
    // }
    // brillig fn foo f1 {
    // b0(v0: Field, v1: Field):
    //     v2 = eq v0, v1
    //     constrain v2 == u1 0
    //     return v0
    // }
    // brillig fn foo f2 {
    //     b0(v0: Field, v1: Field):
    //       v2 = eq v0, v1
    //       constrain v2 == u1 0
    //       return v0
    // }
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::field());
    let main_v1 = builder.add_parameter(Type::field());

    let foo_id = Id::test_new(1);
    let foo = builder.import_function(foo_id);
    let bar_id = Id::test_new(2);
    let bar = builder.import_function(bar_id);

    // Insert multiple calls to the same Brillig function
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    // Interleave a call to a separate Brillig function to make sure that we can call multiple separate Brillig functions
    builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(bar, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.terminate_with_return(vec![]);

    build_basic_foo_with_return(&mut builder, foo_id, true, InlineType::default());
    build_basic_foo_with_return(&mut builder, bar_id, true, InlineType::default());

    let ssa = builder.finish();
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, brillig_functions, _, _) = ssa
        .generate_entry_point_index()
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
    assert_eq!(brillig_functions.len(), 2, "Should only have generated two Brillig functions");

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    assert_eq!(main_opcodes.len(), 6, "Should have four calls to f1 and two calls to f2");

    // We should only have `BrilligCall` opcodes in `main`
    for (i, opcode) in main_opcodes.iter().enumerate() {
        match opcode {
            Opcode::BrilligCall { id, .. } => {
                let expected_id = if i == 3 || i == 5 { 1 } else { 0 };
                let expected_id = BrilligFunctionId(expected_id);
                assert_eq!(*id, expected_id, "Expected an id of {expected_id} but got {id}");
            }
            _ => panic!("Expected only Brillig call opcode"),
        }
    }

    // We have two normal Brillig functions that was called multiple times.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(main_acir.brillig_locations.len(), 2);
    assert!(main_acir.brillig_locations.contains_key(&BrilligFunctionId(0)));
    assert!(main_acir.brillig_locations.contains_key(&BrilligFunctionId(1)));
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
    let ssa = Ssa::from_str(src).unwrap();

    // The Brillig bytecode we insert for the stdlib is hardcoded so we do not need to provide any
    // Brillig artifacts to the ACIR gen pass.
    let (acir_functions, brillig_functions, _, _) = ssa
        .generate_entry_point_index()
        .into_acir(&Brillig::default(), &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
    // We expect two brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    assert_eq!(brillig_functions.len(), 2, "Should only have generated two Brillig functions");

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    check_brillig_calls(&acir_functions[0].brillig_stdlib_func_locations, main_opcodes, 0, 4, 0);

    assert_eq!(main_acir.brillig_locations.len(), 0);
}

// Test that given both hardcoded Brillig directives and calls to normal Brillig functions,
// we generate a single bytecode for the directives and a single bytecode for the normal Brillig calls.
#[test]
fn brillig_stdlib_calls_with_regular_brillig_call() {
    // acir(inline) fn main f0 {
    //     b0(v0: u32, v1: u32, v2: u32):
    //       v4 = div v0, v1
    //       constrain v4 == v2
    //       v5 = call f1(v0, v1)
    //       v6 = call f1(v0, v1)
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
    let foo_id = Id::test_new(0);
    let mut builder = FunctionBuilder::new("main".into(), foo_id);
    let main_v0 = builder.add_parameter(Type::unsigned(32));
    let main_v1 = builder.add_parameter(Type::unsigned(32));
    let main_v2 = builder.add_parameter(Type::unsigned(32));

    let foo_id = Id::test_new(1);
    let foo = builder.import_function(foo_id);

    // Call a primitive operation that uses Brillig
    let v0_div_v1 = builder.insert_binary(main_v0, BinaryOp::Div, main_v1);
    builder.insert_constrain(v0_div_v1, main_v2, None);

    // Insert multiple calls to the same Brillig function
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();
    builder.insert_call(foo, vec![main_v0, main_v1], vec![Type::field()]).to_vec();

    // Call the same primitive operation again
    let v1_div_v2 = builder.insert_binary(main_v1, BinaryOp::Div, main_v2);
    let one = builder.numeric_constant(1u128, NumericType::unsigned(32));
    builder.insert_constrain(v1_div_v2, one, None);

    builder.terminate_with_return(vec![]);

    build_basic_foo_with_return(&mut builder, foo_id, true, InlineType::default());

    let ssa = builder.finish();
    // We need to generate  Brillig artifacts for the regular Brillig function and pass them to the ACIR generation pass.
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, brillig_functions, _, _) = ssa
        .generate_entry_point_index()
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1, "Should only have a `main` ACIR function");
    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(brillig_functions.len(), 3, "Should only have generated three Brillig functions");

    let main_acir = &acir_functions[0];
    let main_opcodes = main_acir.opcodes();
    check_brillig_calls(&main_acir.brillig_stdlib_func_locations, main_opcodes, 1, 4, 2);

    // We have one normal Brillig functions that was called twice.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(main_acir.brillig_locations.len(), 1);
    assert!(main_acir.brillig_locations.contains_key(&BrilligFunctionId(0)));
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

#[test]
fn unchecked_mul_should_not_have_range_check() {
    let src = "
            acir(inline) fn main f0 {
            b0(v0: u32, v1: u32):
                v3 = unchecked_mul v0, v1
                return v3
            }
        ";
    let ssa = Ssa::from_str(src).unwrap();
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (mut acir_functions, _brillig_functions, _, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);

    let opcodes = acir_functions[0].take_opcodes();

    for opcode in opcodes {
        if let Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input }) = opcode {
            assert!(
                input.to_witness().0 <= 1,
                "only input witnesses should have range checks: {opcode:?}"
            );
        }
    }
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
    let ssa = Ssa::from_str(src).unwrap();
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions, _brillig_functions, _, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);

    // Check that no memory opcodes were emitted.
    let main = &acir_functions[0];
    assert!(!main.opcodes().iter().any(|opcode| matches!(opcode, Opcode::MemoryOp { .. })));
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

    let (acir_functions, brillig_functions, _, _) = codegen_acir(
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

    let (acir_functions, _brillig_functions, _, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");

    assert_eq!(acir_functions.len(), 1);
    assert!(acir_functions[0].opcodes().is_empty());
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
    let (acir_functions, brillig_functions, _, _) = ssa
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
    if (lhs + rhs).to_u128() <= u32::MAX as u128 {
        unchecked_operators.push("unchecked_add");
    }
    if (lhs * rhs).to_u128() <= u32::MAX as u128 {
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
    let lhs = FieldElement::from(lhs as u128);
    let rhs = FieldElement::from(rhs as u128);
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u16", &[lhs,rhs]);
    test_operators(&operators, "i16", &[lhs,rhs]);
}

#[test]
fn test_constraint_u8(lhs in 0u8.., rhs in 0u8..) {
    let lhs = FieldElement::from(lhs as u128);
    let rhs = FieldElement::from(rhs as u128);
    let operators = ["constrain ==", "constrain !="];
    test_operators(&operators, "u8", &[lhs,rhs]);
    test_operators(&operators, "i8", &[lhs,rhs]);
}

}
