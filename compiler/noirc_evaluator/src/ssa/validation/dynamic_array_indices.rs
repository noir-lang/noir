use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, Intrinsic},
            value::{Value, ValueId},
        },
        ssa_gen::Ssa,
    },
};

/// Verifies there are no `array_get`, `array_set`, or dynamically-indexed vector
/// intrinsics remaining with dynamic indices where the element type may contain a
/// reference type.
/// This effectively bans dynamic-indexing of arrays with reference elements
/// since we cannot guarantee we optimize references out of the program in that case.
///
/// This pass expects to be run late in the Ssa pipeline such that all array indices
/// used are either constant or derived from an input to the program. E.g. we expect
/// optimizations from inlining, flattening, and mem2reg to already be complete.
pub(crate) fn verify_no_dynamic_indices_to_references(ssa: &Ssa) -> RtResult<()> {
    for function in ssa.functions.values() {
        verify_function(function)?;
    }
    Ok(())
}

fn verify_function(function: &Function) -> RtResult<()> {
    if function.runtime().is_brillig() {
        return Ok(());
    }

    for block in function.reachable_blocks() {
        for instruction in function.dfg[block].instructions() {
            match &function.dfg[*instruction] {
                Instruction::ArrayGet { array, index, .. }
                | Instruction::ArraySet { array, index, .. } => {
                    let array_type = function.dfg.type_of_value(*array);
                    let contains_reference = array_type.contains_reference();

                    if contains_reference && !is_non_dynamic(&function.dfg, *index) {
                        let call_stack = function.dfg.get_instruction_call_stack(*instruction);
                        return Err(RuntimeError::DynamicIndexingWithReference { call_stack });
                    }
                }
                // These vector intrinsics return an element of the vector. If the length or index
                // they select with is dynamic, the returned reference is dynamic too, so we error.
                Instruction::Call { func, arguments } => {
                    let Value::Intrinsic(intrinsic) = function.dfg[*func] else {
                        continue;
                    };
                    let selectors: &[ValueId] = match intrinsic {
                        Intrinsic::VectorRemove => &[arguments[0], arguments[2]],
                        Intrinsic::VectorInsert => &[arguments[2]],
                        Intrinsic::VectorPopFront | Intrinsic::VectorPopBack => &[arguments[0]],
                        _ => continue,
                    };

                    let vector_type = function.dfg.type_of_value(arguments[1]);
                    let contains_reference = vector_type.contains_reference();
                    let is_dynamic =
                        selectors.iter().any(|value| !is_non_dynamic(&function.dfg, *value));

                    if contains_reference && is_dynamic {
                        let call_stack = function.dfg.get_instruction_call_stack(*instruction);
                        return Err(RuntimeError::DynamicIndexingWithReference { call_stack });
                    }
                }
                _ => (),
            }
        }
    }
    Ok(())
}

/// Check if an value is a numeric constant, or a result of an instruction that only uses numeric constant inputs.
fn is_non_dynamic(dfg: &DataFlowGraph, value: ValueId) -> bool {
    // We could check if a non-constant-numeric value is a result of for example a binary an instruction that only
    // takes numeric constant input. However, if we have such a value, it might be a result of an overflowing
    // index expression that we could not simplify at runtime, which means most likely mem2reg could not eliminate
    // the reference allocation either, so even if we classified such indexes as non-dynamic, since they only use
    // known constants, we would just get another obscure error down the line with a less obvious call stack.
    dfg.get_numeric_constant(value).is_some()
}

#[cfg(test)]
mod tests {
    use crate::{errors::RuntimeError, ssa::ssa_gen::Ssa};

    use super::verify_no_dynamic_indices_to_references;

    #[test]
    fn dynamic_array_of_2_mut_bools() {
        // https://github.com/noir-lang/noir/issues/8750
        // fn main(c: u32) -> pub bool {
        //     let b: [&mut bool; 2] = [&mut false, &mut true];
        //     *b[c % 2]
        //}
        let src = r#"
            acir(inline) predicate_pure fn main f0 {
            b0(v0: u32):
                v1 = allocate -> &mut u1
                v2 = allocate -> &mut u1
                v3 = make_array [v1, v2] : [&mut u1; 2]
                v4 = truncate v0 to 1 bits, max_bit_size: 32
                v6 = lt v4, u32 2
                constrain v6 == u1 1, "Index out of bounds"
                v8 = array_get v3, index v4 -> &mut u1
                v9 = load v8 -> u1
                return v9
            }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn no_error_in_brillig() {
        // unconstrained fn main(c: u32) -> pub bool {
        //     let b: [&mut bool; 2] = [&mut false, &mut true];
        //     *b[c % 2]
        //}
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
            b0(v0: u32):
                v1 = allocate -> &mut u1
                v2 = allocate -> &mut u1
                v3 = make_array [v1, v2] : [&mut u1; 2]
                v4 = truncate v0 to 1 bits, max_bit_size: 32
                v6 = lt v4, u32 2
                constrain v6 == u1 1, "Index out of bounds"
                v8 = array_get v3, index v4 -> &mut u1
                v9 = load v8 -> u1
                return v9
            }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(result.is_ok());
    }

    #[test]
    fn error_on_index_overflow() {
        // https://github.com/noir-lang/noir/issues/9853
        // fn main() -> pub bool {
        //     let mut e: [(&mut Field, bool); 1] = [((&mut -1), false)];
        //     e[2147483648].1
        // }
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v2 = make_array [v0, u1 0] : [(&mut Field, u1); 1]
            v5 = unchecked_mul u32 2147483648, u32 2
            v7 = unchecked_add v5, u32 1
            v8 = array_get v2, index v7 -> u1
            return v8
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn dynamic_vector_remove_of_references() {
        // https://github.com/noir-lang/noir-claude/issues/1363
        // fn main(i: u32) -> pub Field {
        //     let mut x = 10;
        //     let mut y = 20;
        //     let v: [&mut Field] = [&mut x, &mut y].as_vector();
        //     let (_v2, removed) = v.remove(i);
        //     y = 77;
        //     *removed
        // }
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v4, v5, v6 = call vector_remove(u32 2, v3, v0) -> (u32, [&mut Field], &mut Field)
            store Field 77 at v2
            v7 = load v6 -> Field
            return v7
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn dynamic_vector_insert_of_references() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v4, v5 = call vector_insert(u32 2, v3, v0, v1) -> (u32, [&mut Field])
            return u1 0
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn dynamic_length_vector_pop_back_of_references() {
        // A `pop_back` reads element `length - 1`; a dynamic length is therefore a dynamic
        // reference selection even though the popped position is "the last element".
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v4, v5, v6 = call vector_pop_back(v0, v3) -> (u32, [&mut Field], &mut Field)
            v7 = load v6 -> Field
            return v7
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn dynamic_length_vector_pop_front_of_references() {
        // A `pop_front` reads element `0`, but cannot be resolved without a constant length,
        // so a dynamic length is rejected here too.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v6, v4, v5 = call vector_pop_front(v0, v3) -> (&mut Field, u32, [&mut Field])
            v7 = load v6 -> Field
            return v7
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn constant_length_vector_pop_back_of_references_is_allowed() {
        // A constant length can be resolved by simplification, so it must not be rejected.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v4, v5, v6 = call vector_pop_back(u32 2, v3) -> (u32, [&mut Field], &mut Field)
            v7 = load v6 -> Field
            return v7
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(result.is_ok());
    }

    #[test]
    fn constant_vector_remove_of_references_is_allowed() {
        // A constant remove index can be resolved by mem2reg, so it must not be rejected.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store Field 10 at v1
            store Field 20 at v2
            v3 = make_array [v1, v2] : [&mut Field]
            v4, v5, v6 = call vector_remove(u32 2, v3, u32 1) -> (u32, [&mut Field], &mut Field)
            v7 = load v6 -> Field
            return v7
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = verify_no_dynamic_indices_to_references(&ssa);
        assert!(result.is_ok());
    }
}
