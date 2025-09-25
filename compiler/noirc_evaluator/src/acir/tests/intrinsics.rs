use crate::acir::tests::ssa_to_acir_program;

#[test]
fn slice_push_back_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_push_back(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_push_back(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_push_front_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_push_front(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_push_front(u32 1, v7, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_pop_back_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_pop_back(u32 1, v7) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_pop_back(u32 1, v7) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_pop_front_not_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_pop_front(u32 1, v7) -> (Field, [Field; 2], u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_pop_front(u32 1, v7) -> (Field, [Field; 2], u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_eq!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_insert_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10 = call slice_insert(u32 1, v7, u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10 = call slice_insert(u32 1, v7, u32 1, Field 1, v4) -> (u32, [(Field, [Field; 2])])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_ne!(program_side_effects, program_no_side_effects);
}

#[test]
fn slice_remove_affected_by_predicate() {
    let src_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        enable_side_effects v1
        v9, v10, v11, v12 = call slice_remove(u32 1, v7, u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";
    let src_no_side_effects = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u1):
        v4 = make_array [Field 2, Field 3] : [Field; 2]
        v5 = make_array [Field 1, v4] : [(Field, [Field; 2])]
        v7 = array_set v5, index v0, value Field 4
        v9, v10, v11, v12 = call slice_remove(u32 1, v7, u32 1) -> (u32, [(Field, [Field; 2])], Field, [Field; 2])
        return
    }
    ";

    let program_side_effects = ssa_to_acir_program(&src_side_effects);
    let program_no_side_effects = ssa_to_acir_program(&src_no_side_effects);
    assert_ne!(program_side_effects, program_no_side_effects);
}
