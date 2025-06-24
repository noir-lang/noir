//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! values with zeroed values of the appropriate type. If the block has successors
//! those successors will also be considered unreachable if they are dominated
//! by that block.
use std::sync::Arc;

use fxhash::FxHashSet as HashSet;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function,
        instruction::Instruction, post_order::PostOrder, types::Type, value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn remove_unreachable_instructions(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_unreachable_instructions();
        }
        self
    }
}

impl Function {
    fn remove_unreachable_instructions(&mut self) {
        println!("remove_unreachable_instructions from {}", self.id());
        // Iterate each block in reverse post order = forward order
        let cfg = ControlFlowGraph::with_function(self);
        let mut block_order = PostOrder::with_cfg(&cfg).into_vec();
        // Start with the entry, process all blocks before their successors.
        block_order.reverse();

        // The current block we are currently processing
        let mut current_block_id = None;

        // Whether the current block instructions were determined to be unreachable
        let mut current_block_instructions_are_unreachable = false;

        // This is the final set of blocks that we concluded have some unreachable instructions.
        // At the end we'll zero out their terminators.
        let mut unreachable_blocks = HashSet::default();

        self.blocks_optimization(block_order, |context| {
            let block_id = context.block_id;

            if current_block_id != Some(block_id) {
                current_block_id = Some(block_id);
                let has_predecessors = cfg.predecessors(block_id).len() > 0;
                current_block_instructions_are_unreachable = has_predecessors
                    && cfg
                        .predecessors(block_id)
                        .all(|block_id| unreachable_blocks.contains(&block_id));

                if current_block_instructions_are_unreachable {
                    unreachable_blocks.insert(block_id);
                }
            }

            if current_block_instructions_are_unreachable {
                context.remove_current_instruction();
                return;
            }

            let instruction = context.instruction();
            let is_unreachable = match instruction {
                Instruction::Constrain(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    lhs_constant != rhs_constant
                }
                Instruction::ConstrainNotEqual(lhs, rhs, _) => {
                    let Some(lhs_constant) = context.dfg.get_numeric_constant(*lhs) else {
                        return;
                    };
                    let Some(rhs_constant) = context.dfg.get_numeric_constant(*rhs) else {
                        return;
                    };
                    lhs_constant == rhs_constant
                }
                _ => false,
            };

            if is_unreachable {
                unreachable_blocks.insert(block_id);
                current_block_instructions_are_unreachable = true;
            }
        });

        for block_id in unreachable_blocks {
            let mut terminator = self.dfg[block_id].take_terminator();
            terminator.map_values_mut(|value_id| {
                let typ = self.dfg.type_of_value(value_id);
                dummy_ref(self, block_id, typ)
            });
            self.dfg[block_id].set_terminator(terminator);
        }
    }
}

/// Pretend that we have a value for the terminator by allocating a reference and loading it.
///
/// We will never store to this reference, so it would be an error to actually execute the instruction,
/// but it should prevent the SSA optimizations from simplifying `JmpIf` instructions into `Jmp` if we
/// use some kind of constant value instead, which would alter the CFG and cause other issues,
/// such as infinite loops, that other passes would have difficulty dealing with.
///
/// The alternative to this is to have a new kind of _unreachable_ terminator instruction.
fn dummy_ref(function: &mut Function, block_id: BasicBlockId, typ: Type) -> ValueId {
    // Pretend that we have a reference for the appropriate type.
    let instruction = Instruction::Allocate;
    let reference_type = Type::Reference(Arc::new(typ.clone()));
    let reference_id = function
        .dfg
        .insert_instruction_and_results(
            instruction,
            block_id,
            Some(vec![reference_type]),
            CallStackId::root(),
        )
        .first();

    // Load the reference. We should never execute this instruction, because the code is unreachable.
    let instruction = Instruction::Load { address: reference_id };
    function
        .dfg
        .insert_instruction_and_results(instruction, block_id, Some(vec![typ]), CallStackId::root())
        .first()
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    #[test]
    fn removes_unreachable_instructions_in_block_for_constrain_equal() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            return v5
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v3 = allocate -> &mut u1
            v4 = load v3 -> u1
            return v4
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_in_block_for_constrain_not_equal() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 != u1 0, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            return v5
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 != u1 0, "Index out of bounds"
            v2 = allocate -> &mut u1
            v3 = load v2 -> u1
            return v3
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_normal_order() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            jmp b1(v5)
          b1(v6: u1):
            v7 = add v6, u1 1
            jmp b2(v7)
          b2(v8: u1):
            v9 = add v8, u1 1
            return v9
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v5 = allocate -> &mut u1
            v6 = load v5 -> u1
            jmp b1(v6)
          b1(v0: u1):
            v7 = allocate -> &mut u1
            v8 = load v7 -> u1
            jmp b2(v8)
          b2(v1: u1):
            v9 = allocate -> &mut u1
            v10 = load v9 -> u1
            return v10
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_different_order() {
        // This is the same as `removes_unreachable_instructions_from_dominated_blocks_normal_order`
        // except that the blocks are in a different order.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v4 = array_get v0, index u32 0 -> &mut u1
            v5 = load v4 -> u1
            jmp b2(v5)
          b1(v8: u1):
            v9 = add v8, u1 1
            return v9
          b2(v6: u1):
            v7 = add v6, u1 1
            jmp b1(v7)
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [] : [&mut u1; 0]
            constrain u1 0 == u1 1, "Index out of bounds"
            v5 = allocate -> &mut u1
            v6 = load v5 -> u1
            jmp b2(v6)
          b1(v0: u1):
            v9 = allocate -> &mut u1
            v10 = load v9 -> u1
            return v10
          b2(v1: u1):
            v7 = allocate -> &mut u1
            v8 = load v7 -> u1
            jmp b1(v8)
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_dominated_blocks_transitively() {
        // This tests that if a block has an unreachable instruction,
        // all of its successors that are dominated by it are also unreachable,
        // and that is applied recursively.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            v1 = add Field 1, Field 2
            jmpif u1 0 then: b2, else: b3
          b2():
            v2 = add Field 1, Field 2
            jmp b1()
          b3():
            v3 = add Field 1, Field 2
            return v3
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            jmp b1()
          b3():
            return Field 0
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_following_block_with_no_instructions() {
        // This tests that if a block is determined to be unreachable,
        // a dominated block that has no instructions also gets its terminator zeroed out.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            jmp b2()
          b2():
            v2 = add Field 1, Field 2
            return v2
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b1()
          b1():
            jmp b2()
          b2():
            v4 = add Field 1, Field 2
            return v4
        }
        "#);
    }

    #[test]
    fn does_not_zeroes_terminator_of_non_dominated_block() {
        // Here both b1 and b4 are successors of b3, but both are not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            v2 = add Field 1, Field 2
            jmp b2(v2)
          b2():
            jmpif u1 0 then: b3, else: b4
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmpif u1 0 then: b4, else: b1
          b4():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_zeroes_terminator_of_non_dominated_block_2() {
        // Here b3 is a successof of b2 but is not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b3()
          b3():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_zeroes_terminator_of_non_dominated_block_3() {
        // Here b4 is a transitive successor of b2 but is not dominated by it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            jmp b4()
          b4():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_zeroes_terminator_of_non_dominated_block_4() {
        // Here b5 is a transitive successor of b2, but is not dominated by it
        // (it's a transitive successof of b1)
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1()
          b1():
            jmpif u1 0 then: b2, else: b3
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn removes_block_that_is_unreachable_when_all_of_its_predecessors_are_unreachable() {
        // Here b4 won't be conisdered unreachable when we find that b2 or b3 are unreachable,
        // because neither dominate it, but it will still not show up in the final SSA
        // because no block will be able to reach it.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b4():
            return Field 1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b3():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b4()
          b4():
            return Field 1
        }
        "#);
    }
}
