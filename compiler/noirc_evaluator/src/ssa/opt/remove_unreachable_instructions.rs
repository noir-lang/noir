//! This module defines an SSA pass to remove instructions that are unreachable.
//! For example, if an instruction in a block is `constrain u1 0 == u1 1`,
//! any subsequent instructions in that block will never be executed. This pass
//! then removes those subsequent instructions and replaces the block's terminator
//! values with zeroed values of the appropriate type. If the block has successors
//! whose predecessors are that block only, those successors will also be unreachable
//! so the same treatment is applied to them.
use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use fxhash::FxHashSet as HashSet;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId, cfg::ControlFlowGraph, function::Function,
        instruction::Instruction, types::Type, value::ValueId,
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
        let cfg = ControlFlowGraph::with_function(self);

        let mut current_block_id = None;
        let mut current_block_instructions_are_unreachable = false;
        let mut unreachable_blocks = HashSet::default();

        self.simple_reachable_blocks_optimization(|context| {
            let block_id = context.block_id;

            if current_block_id != Some(block_id) {
                current_block_id = Some(block_id);
                current_block_instructions_are_unreachable = unreachable_blocks.contains(&block_id);

                if current_block_instructions_are_unreachable {
                    add_successors(block_id, &cfg, &mut unreachable_blocks);
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
                current_block_instructions_are_unreachable = true;
                unreachable_blocks.insert(block_id);

                add_successors(block_id, &cfg, &mut unreachable_blocks);
            }
        });

        for block_id in unreachable_blocks {
            let mut terminator = self.dfg[block_id].take_terminator();
            terminator.map_values_mut(|value_id| {
                let typ = self.dfg.type_of_value(value_id);
                zeroed_value(self, block_id, &typ)
            });
            self.dfg[block_id].set_terminator(terminator);
        }
    }
}

/// Adds all of a block's successors to the `blocks` set, if each of those successors
/// have the given block as their only predecessor.
fn add_successors(
    block_id: BasicBlockId,
    cfg: &ControlFlowGraph,
    blocks: &mut HashSet<BasicBlockId>,
) {
    for successor in cfg.successors(block_id) {
        let successor_predecessors = cfg.predecessors(successor).collect::<Vec<_>>();
        if successor_predecessors.len() == 1 && successor_predecessors[0] == block_id {
            blocks.insert(successor);
        }
    }
}

fn zeroed_value(function: &mut Function, block_id: BasicBlockId, typ: &Type) -> ValueId {
    match typ {
        Type::Numeric(numeric_type) => {
            function.dfg.make_constant(FieldElement::zero(), *numeric_type)
        }
        Type::Array(element_types, len) => {
            let mut array = im::Vector::new();
            for _ in 0..*len {
                for typ in element_types.iter() {
                    array.push_back(zeroed_value(function, block_id, typ));
                }
            }
            let instruction = Instruction::MakeArray { elements: array, typ: typ.clone() };
            let stack = CallStackId::root();
            function.dfg.insert_instruction_and_results(instruction, block_id, None, stack).first()
        }
        Type::Slice(_) => {
            let array = im::Vector::new();
            let instruction = Instruction::MakeArray { elements: array, typ: typ.clone() };
            let stack = CallStackId::root();
            function.dfg.insert_instruction_and_results(instruction, block_id, None, stack).first()
        }
        Type::Reference(element_type) => {
            let instruction = Instruction::Allocate;
            let reference_type = Type::Reference(Arc::new((**element_type).clone()));
            function
                .dfg
                .insert_instruction_and_results(
                    instruction,
                    block_id,
                    Some(vec![reference_type]),
                    CallStackId::root(),
                )
                .first()
        }
        Type::Function => {
            // We can have the function return itself. It's fine because the terminator is unreachable anyway.
            function.dfg.import_function(function.id())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    #[test]
    fn removes_unreachable_instructions_in_block() {
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
            return u1 0
        }
        "#);
    }

    #[test]
    fn removes_unreachable_instructions_from_successors() {
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
            jmp b1(u1 0)
          b1(v0: u1):
            jmp b2(u1 0)
          b2(v1: u1):
            return u1 0
        }
        "#);
    }

    #[test]
    fn does_not_remove_unreachable_instructions_from_successor_if_they_have_other_predecessors() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmpif u1 0 then: b1, else: b2
          b1():
            constrain u1 0 == u1 1, "Index out of bounds"
            jmp b2()
          b2():
            v1 = add Field 1, Field 2
            return v1
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_unreachable_instructions();
        assert_normalized_ssa_equals(ssa, src);
    }
}
