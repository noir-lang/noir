//! Validates the SSA dominance property: every use of a value must be dominated
//! by its definition. This catches bugs where a pass creates references to values
//! in unreachable blocks or in blocks that don't dominate the use site.

use rustc_hash::FxHashMap as HashMap;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dom::DominatorTree,
    function::Function,
    instruction::InstructionId,
    value::{Value, ValueId},
};

/// Validates that every value referenced by instructions and terminators in reachable blocks
/// satisfies the SSA dominance property: the block where a value is defined must dominate
/// every block where it is used. For uses within the same block as the definition,
/// the defining instruction must appear before the using instruction.
pub(crate) fn validate_value_definitions(function: &Function) {
    let dfg = &function.dfg;
    let reachable_blocks = function.reachable_blocks();
    let dom_tree = DominatorTree::with_function(function);

    // Build a map from ValueId -> (defining block, position within block).
    // Block parameters have position None (defined at block entry, before all instructions).
    // Instruction results have position Some(index).
    let mut value_def_site: HashMap<ValueId, (BasicBlockId, Option<usize>)> = HashMap::default();

    for &block_id in &reachable_blocks {
        for param in dfg.block_parameters(block_id) {
            value_def_site.insert(*param, (block_id, None));
        }
        for (pos, instruction_id) in dfg[block_id].instructions().iter().enumerate() {
            for result in dfg.instruction_results(*instruction_id) {
                value_def_site.insert(*result, (block_id, Some(pos)));
            }
        }
    }

    // Check each use in every reachable block
    for &block_id in &reachable_blocks {
        let block = &dfg[block_id];

        for (use_pos, instruction_id) in block.instructions().iter().enumerate() {
            dfg[*instruction_id].for_each_value(|value_id| {
                assert_value_dominates_use(
                    function,
                    value_id,
                    block_id,
                    Some((*instruction_id, use_pos)),
                    &value_def_site,
                    &dom_tree,
                );
            });
        }

        if let Some(terminator) = block.terminator() {
            terminator.for_each_value(|value_id| {
                assert_value_dominates_use(
                    function,
                    value_id,
                    block_id,
                    None,
                    &value_def_site,
                    &dom_tree,
                );
            });
        }
    }
}

/// Returns true if a value is "free" — i.e., it doesn't need a prior definition
/// because it's a constant, function reference, intrinsic, foreign function, or global.
fn is_free_value(function: &Function, value_id: ValueId) -> bool {
    let dfg = &function.dfg;
    if dfg.is_global(value_id) {
        return true;
    }
    matches!(
        &dfg[value_id],
        Value::NumericConstant { .. }
            | Value::Function(_)
            | Value::Intrinsic(_)
            | Value::ForeignFunction(_)
            | Value::Global(_)
    )
}

/// Assert that a value's definition dominates a use site.
///
/// `use_site` is `Some((instruction_id, position))` for instruction operands, or `None` for
/// terminator operands. When a use and definition are in the same block, the definition's
/// instruction position must be strictly less than the use position.
fn assert_value_dominates_use(
    function: &Function,
    value_id: ValueId,
    use_block: BasicBlockId,
    use_site: Option<(InstructionId, usize)>,
    value_def_site: &HashMap<ValueId, (BasicBlockId, Option<usize>)>,
    dom_tree: &DominatorTree,
) {
    if is_free_value(function, value_id) {
        return;
    }

    let dfg = &function.dfg;

    let Some(&(def_block, def_pos)) = value_def_site.get(&value_id) else {
        let use_desc = match use_site {
            Some((instr_id, _)) => format!("instruction {:?}", &dfg[instr_id]),
            None => "terminator".to_string(),
        };
        panic!(
            "Use-before-def in function {} ({}): \
             {use_desc} in block {use_block} references value {value_id} \
             which is not defined in any reachable block.\n\
             Value info: {:?}",
            function.name(),
            function.id(),
            &dfg[value_id],
        );
    };

    if def_block == use_block {
        // Same block: definition must come before use.
        // Block parameters (def_pos = None) are always before any instruction.
        if let Some(def_position) = def_pos {
            let use_position = match use_site {
                Some((_, pos)) => pos,
                // Terminators are after all instructions, so always valid
                None => return,
            };
            if def_position >= use_position {
                let (instr_id, _) = use_site.unwrap();
                panic!(
                    "Use-before-def in function {} ({}): \
                     instruction {:?} at position {use_position} in block {use_block} uses value {value_id} \
                     which is defined at position {def_position} in the same block.\n\
                     Value info: {:?}",
                    function.name(),
                    function.id(),
                    &dfg[instr_id],
                    &dfg[value_id],
                );
            }
        }
    } else {
        // Different blocks: the definition block must dominate the use block.
        if !dom_tree.dominates_helper(def_block, use_block) {
            let use_desc = match use_site {
                Some((instr_id, _)) => format!("instruction {:?}", &dfg[instr_id]),
                None => "terminator".to_string(),
            };
            panic!(
                "Use-before-def in function {} ({}): \
                 {use_desc} in block {use_block} references value {value_id} \
                 defined in block {def_block} which does not dominate block {use_block}.\n\
                 Value info: {:?}",
                function.name(),
                function.id(),
                &dfg[value_id],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            function::{FunctionId, RuntimeType},
            instruction::BinaryOp,
            types::{NumericType, Type},
        },
        ssa_gen::Ssa,
    };

    use super::validate_value_definitions;

    #[test]
    fn valid_simple_function() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = add v0, v1
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn valid_value_used_in_dominated_block() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = add v0, u32 1
            jmp b1()
          b1():
            return v1
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn valid_block_parameter_dominates_body() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(v0)
          b1(v1: u32):
            v2 = add v1, u32 1
            return v2
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    #[test]
    fn valid_diamond_cfg() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            jmpif v0 then: b1, else: b2
          b1():
            v2 = add v1, u32 1
            jmp b3()
          b2():
            v3 = add v1, u32 2
            jmp b3()
          b3():
            return
        }
        ";
        let _ = Ssa::from_str(src).unwrap();
    }

    /// Helper to build a diamond CFG where a value defined in b1 is illegally used in b2.
    /// b1 does NOT dominate b2.
    ///
    /// brillig fn main f0 {
    ///   b0(v0: u1, v_param: u32):
    ///     jmpif v0 then: b1, else: b2
    ///   b1():
    ///     v_in_b1 = add v_param, u32 1
    ///     ...
    ///   b2():
    ///     <uses v_in_b1 illegally>
    ///     ...
    /// }
    fn build_diamond_with_dominance_violation(use_in_terminator: bool) -> Ssa {
        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let v0 = builder.add_parameter(Type::bool());
        let v_param = builder.add_parameter(Type::unsigned(32));
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        builder.terminate_with_jmpif(v0, b1, b2);

        // b1: define v_in_b1
        builder.switch_to_block(b1);
        let one = builder.numeric_constant(1u128, NumericType::unsigned(32));
        let v_in_b1 = builder.insert_binary(v_param, BinaryOp::Add { unchecked: true }, one);
        builder.terminate_with_jmp(b3, vec![]);

        // b2: illegally use v_in_b1 (b1 doesn't dominate b2)
        builder.switch_to_block(b2);
        if use_in_terminator {
            builder.terminate_with_jmp(b3, vec![v_in_b1]);
        } else {
            let _bad = builder.insert_binary(v_in_b1, BinaryOp::Add { unchecked: true }, one);
            builder.terminate_with_jmp(b3, vec![]);
        }

        // b3
        builder.switch_to_block(b3);
        if use_in_terminator {
            let _p = builder.add_block_parameter(b3, Type::unsigned(32));
        }
        builder.terminate_with_return(vec![]);

        builder.finish()
    }

    #[test]
    #[should_panic(expected = "does not dominate")]
    fn catches_use_of_value_from_non_dominating_block() {
        let ssa = build_diamond_with_dominance_violation(false);
        validate_value_definitions(ssa.main());
    }

    #[test]
    #[should_panic(expected = "does not dominate")]
    fn catches_terminator_use_from_non_dominating_block() {
        let ssa = build_diamond_with_dominance_violation(true);
        validate_value_definitions(ssa.main());
    }

    #[test]
    #[should_panic(expected = "not defined in any reachable block")]
    fn catches_use_of_value_from_unreachable_block() {
        let main_id = FunctionId::new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let v_param = builder.add_parameter(Type::unsigned(32));
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        // b0 jumps directly to b2, making b1 unreachable
        builder.terminate_with_jmp(b2, vec![]);

        // b1 (unreachable): define a value
        builder.switch_to_block(b1);
        let one = builder.numeric_constant(1u128, NumericType::unsigned(32));
        let v_unreachable = builder.insert_binary(v_param, BinaryOp::Add { unchecked: true }, one);
        builder.terminate_with_jmp(b2, vec![]);

        // b2: use the value from unreachable b1
        builder.switch_to_block(b2);
        let _bad = builder.insert_binary(v_unreachable, BinaryOp::Add { unchecked: true }, one);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        validate_value_definitions(ssa.main());
    }
}
