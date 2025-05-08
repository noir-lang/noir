use acvm::{FieldElement, acir::circuit::ExpressionWidth};

use crate::{
    acir::GeneratedAcir,
    brillig::BrilligOptions,
    ssa::{ir::instruction::Intrinsic, ssa_gen::Ssa},
};

#[test]
fn slice_push_back_not_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SlicePushBack.to_string(),
        ", Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SlicePushBack.to_string(),
        ", Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        false,
    )[0];
    assert_eq!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_eq!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

#[test]
fn slice_push_front_not_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SlicePushFront.to_string(),
        ", Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SlicePushFront.to_string(),
        ", Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        false,
    )[0];
    assert_eq!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_eq!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

#[test]
fn slice_pop_back_not_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SlicePopBack.to_string(),
        ") -> (u32, [(Field, [Field; 2])], Field, [Field; 2])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SlicePopBack.to_string(),
        ") -> (u32, [(Field, [Field; 2])], Field, [Field; 2])",
        false,
    )[0];
    assert_eq!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_eq!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

#[test]
fn slice_pop_front_not_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SlicePopFront.to_string(),
        ") -> (Field, [Field; 2], u32, [(Field, [Field; 2])])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SlicePopFront.to_string(),
        ") -> (Field, [Field; 2], u32, [(Field, [Field; 2])])",
        false,
    )[0];
    assert_eq!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_eq!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

#[test]
fn slice_insert_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SliceInsert.to_string(),
        ", u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10",
        &Intrinsic::SliceInsert.to_string(),
        ", u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])",
        false,
    )[0];
    assert_ne!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_ne!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

#[test]
fn slice_remove_affected_by_predicate() {
    let func_with_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SliceRemove.to_string(),
        ", u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])",
        true,
    )[0];
    let func_no_pred = &get_slice_intrinsic_acir(
        "v9, v10, v11, v12",
        &Intrinsic::SliceRemove.to_string(),
        ", u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])",
        false,
    )[0];
    assert_ne!(func_with_pred.current_witness_index(), func_no_pred.current_witness_index());
    assert_ne!(func_with_pred.opcodes(), func_no_pred.opcodes());
}

/// Helper method to set up the SSA for unit tests on whether slice intrinsics
/// are affected by the ACIR gen predicate.
fn get_slice_intrinsic_src(
    return_values: &str,
    intrinsic_name: &str,
    args_and_return_type: &str,
    with_side_effects: bool,
) -> String {
    let side_effects = if with_side_effects { "enable_side_effects v1\n" } else { "" };
    format!(
        "
    acir(inline) predicate_pure fn main f0 {{
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        {side_effects}
        {return_values} = call {intrinsic_name}(u32 1, v7{args_and_return_type}
        return
    }}
    "
    )
}

/// Helper for fetching the ACIR for the SSA specified in [get_slice_intrinsic_src].
/// This helper assumes we have a single main function we are testing against.
fn get_slice_intrinsic_acir(
    return_values: &str,
    intrinsic_name: &str,
    args_and_return_type: &str,
    with_side_effects: bool,
) -> Vec<GeneratedAcir<FieldElement>> {
    let src = get_slice_intrinsic_src(
        return_values,
        intrinsic_name,
        args_and_return_type,
        with_side_effects,
    );
    let ssa = Ssa::from_str(&src).unwrap();
    let brillig = ssa.to_brillig(&BrilligOptions::default());

    let (acir_functions_with_pred, _brillig_functions, _, _) = ssa
        .into_acir(&brillig, &BrilligOptions::default(), ExpressionWidth::default())
        .expect("Should compile manually written SSA into ACIR");
    acir_functions_with_pred
}
