/// Expands array accesses in Brillig to include explicit out of bounds (OOB) checks.
///
/// In the Brillig runtime, array accesses are treated as pointer accesses and thus are unprotected
/// in isolation. For example, if we have an array access that is out of bounds, but there is memory
/// declared for other purposes after the array pointer, the bytecode will look in that unrelated memory.
/// Thus, in order to keep array accesses safe we have separate OOB checks.
///
/// In order to maintain a simple initial SSA generation, we simply inject these checks
/// as part of our SSA compilation flow.
use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, ConstrainError, Instruction},
        types::{NumericType, Type},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn expand_array_oob_checks(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.expand_array_oob_checks();
        }
        self
    }
}

impl Function {
    pub(crate) fn expand_array_oob_checks(&mut self) {
        // This check should only be run over Brillig runtimes
        if self.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction = context.instruction();
            let block_id = context.block_id;

            let (Instruction::ArrayGet { array, index, .. }
            | Instruction::ArraySet { array, index, .. }) = instruction
            else {
                return;
            };

            let array_typ = context.dfg.type_of_value(*array);
            let length = match array_typ {
                Type::Array(elements, length) => elements.len() as u32 * length,
                _ => {
                    // If we do not have an array length it means we have a slice, for which we should
                    // always have separate access checks against the dynamic length in the initial SSA
                    return;
                }
            };

            let index = *index;
            let length = context.dfg.make_constant(length.into(), NumericType::length_type());

            let is_offset_out_of_bounds =
                Instruction::Binary(Binary { lhs: index, operator: BinaryOp::Lt, rhs: length });
            let is_offset_out_of_bounds = context
                .dfg
                .insert_instruction_and_results(
                    is_offset_out_of_bounds,
                    block_id,
                    None,
                    context.call_stack_id,
                )
                .first();

            let true_const = context.dfg.make_constant(true.into(), NumericType::bool());

            let assert_message = Some(ConstrainError::from("Index out of bounds".to_owned()));
            let constrain =
                Instruction::Constrain(is_offset_out_of_bounds, true_const, assert_message);
            context.dfg.insert_instruction_and_results(
                constrain,
                block_id,
                None,
                context.call_stack_id,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    fn check_acir_unchanged(src: &str) {
        let acir_src = &src.replace("brillig", "acir");
        let acir_ssa = Ssa::from_str(acir_src).unwrap();
        let ssa = acir_ssa.expand_array_oob_checks();
        assert_normalized_ssa_equals(ssa, acir_src);
    }

    #[test]
    fn array_get_oob_constant_index_brillig() {
        let src = r"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v3, index u32 10 -> Field
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            constrain u1 0 == u1 1, "Index out of bounds"
            v7 = array_get v3, index u32 10 -> Field
            return
        }
        "#);
    }

    #[test]
    fn array_get_in_bounds_constant_index_brillig() {
        let src = r"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v3, index u32 2 -> Field
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();

        // The always true constrain is expected to be simplified out
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v3, index u32 2 -> Field
            return
        }
        "#);
    }

    #[test]
    fn array_set_oob_constant_index_brillig() {
        let src = r"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v6 = array_set v3, index u32 10, value Field 5
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            constrain u1 0 == u1 1, "Index out of bounds"
            v8 = array_set v3, index u32 10, value Field 5
            return
        }
        "#);
    }

    #[test]
    fn array_get_oob_dynamic_index_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v4, index v0 -> Field
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v6 = lt v0, u32 3
            constrain v6 == u1 1, "Index out of bounds"
            v8 = array_get v4, index v0 -> Field
            return
        }
        "#);
    }

    #[test]
    fn array_set_oob_dynamic_index_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v6 = array_set v4, index v0, value Field 5
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v6 = lt v0, u32 3
            constrain v6 == u1 1, "Index out of bounds"
            v9 = array_set v4, index v0, value Field 5
            return
        }
        "#);
    }

    #[test]
    fn array_get_on_composite_array() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = make_array [Field 2] : [Field; 1]
            v4 = make_array [Field 3] : [Field; 1]
            v6 = make_array [Field 1, v2, v4] : [(Field, [Field; 1], [Field; 1]); 1]
            v7 = make_array [v6] : [[(Field, [Field; 1], [Field; 1]); 1]; 1]
            v9 = sub v0, u32 3
            v10 = array_get v7, index v9 -> [(Field, [Field; 1], [Field; 1]); 1]
            inc_rc v10
            v12 = array_get v10, index u32 0 -> Field
            v14 = array_get v10, index u32 1 -> [Field; 1]
            v16 = array_get v10, index u32 2 -> [Field; 1]
            return
        }
        ";
        check_acir_unchanged(src);

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_array_oob_checks();
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = make_array [Field 2] : [Field; 1]
            v4 = make_array [Field 3] : [Field; 1]
            v6 = make_array [Field 1, v2, v4] : [(Field, [Field; 1], [Field; 1]); 1]
            v7 = make_array [v6] : [[(Field, [Field; 1], [Field; 1]); 1]; 1]
            v9 = sub v0, u32 3
            v11 = eq v9, u32 0
            constrain v9 == u32 0, "Index out of bounds"
            v12 = array_get v7, index v9 -> [(Field, [Field; 1], [Field; 1]); 1]
            inc_rc v12
            v13 = array_get v12, index u32 0 -> Field
            v15 = array_get v12, index u32 1 -> [Field; 1]
            v17 = array_get v12, index u32 2 -> [Field; 1]
            return
        }
        "#);
    }
}
