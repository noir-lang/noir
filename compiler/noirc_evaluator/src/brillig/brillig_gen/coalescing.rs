//! Pre-codegen block parameter coalescing for Brillig.
//!
//! This module analyzes `Jmp` terminators to determine which arguments can share
//! a register with their destination block parameter. When an argument is coalesced
//! with a parameter, the instruction defining the argument writes directly to the
//! parameter's register, eliminating the mov at the jmp site.

use crate::ssa::ir::{
    function::Function,
    instruction::TerminatorInstruction,
    value::{Value, ValueId},
};

use rustc_hash::FxHashMap as HashMap;

use super::variable_liveness::VariableLiveness;

/// Maps SSA ValueIds to partners whose register they should reuse.
///
/// An entry `x -> y` means "when defining x, reuse y's register."
/// This covers two cases:
/// - **Arg-side** (`arg -> param`): an instruction result defined in the same block as the jmp
///   writes directly to the parameter's register.
/// - **Param-side** (`param -> arg`): a block parameter reuses the register of an already-allocated
///   value (block param passthrough, cross-block instruction result).
#[derive(Default)]
pub(crate) struct CoalescingMap {
    coalesced: HashMap<ValueId, ValueId>,
}

impl CoalescingMap {
    /// Analyze all `Jmp` terminators in the function and build the coalescing map.
    ///
    /// For each `(argument, parameter)` pair at a jmp, we attempt to coalesce in one
    /// of two directions:
    /// - **Arg-side**: if the arg is an instruction result defined in the source block,
    ///   record `arg -> param` so the instruction writes to the param's register.
    /// - **Param-side**: if the arg is a block parameter or instruction from another block,
    ///   record `param -> arg` so the param reuses the arg's register.
    pub(crate) fn from_function(func: &Function, liveness: &VariableLiveness) -> Self {
        let mut coalesced = HashMap::default();
        let cfg = liveness.cfg();

        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];
            let Some(terminator) = block.terminator() else {
                continue;
            };

            let TerminatorInstruction::Jmp { destination, arguments, .. } = terminator else {
                continue;
            };

            let dest_block = &func.dfg[*destination];
            let params = dest_block.parameters();
            let dest_live_in = liveness.get_live_in(destination);
            let instructions = func.dfg[block_id].instructions();

            for (arg, param) in arguments.iter().zip(params.iter()) {
                if arg == param {
                    continue;
                }

                // If arg or param is already in the map, skip to avoid conflicts.
                if coalesced.contains_key(arg) || coalesced.contains_key(param) {
                    continue;
                }

                match &func.dfg[*arg] {
                    Value::Instruction { instruction: defining_inst, .. }
                        if instructions.iter().any(|inst_id| inst_id == defining_inst) =>
                    {
                        // Arg-side: instruction defined in this block.
                        let def_pos = instructions
                            .iter()
                            .position(|inst_id| inst_id == defining_inst)
                            .unwrap();

                        // If arg is live-in to the destination block and the destination has
                        // other predecessors, we must not coalesce. When the destination is a
                        // loop header (or merge point), other predecessors will write different
                        // values to param's register on subsequent iterations, destroying arg's
                        // value while it is still needed.
                        if dest_live_in.contains(arg) && cfg.predecessors(*destination).count() > 1
                        {
                            continue;
                        }

                        let live_in = liveness.get_live_in(&block_id);

                        if !live_in.contains(param) {
                            // param is not live-in to the source block, so the defining instruction
                            // can safely write to param's register without clobbering anything.
                            coalesced.insert(*arg, *param);
                            continue;
                        }

                        // Check if param is used in the defining instruction or any subsequent instruction.
                        let mut param_used_at_or_after = false;

                        for inst_id in &instructions[def_pos..] {
                            let instruction = &func.dfg[*inst_id];
                            let mut found = false;
                            instruction.for_each_value(|v| {
                                if v == *param {
                                    found = true;
                                }
                            });
                            if found {
                                param_used_at_or_after = true;
                                break;
                            }
                        }

                        // Also check if param is used in the terminator as an argument (not as a
                        // destination parameter — those are the write side).
                        if !param_used_at_or_after
                            && let Some(term) = func.dfg[block_id].terminator()
                        {
                            term.for_each_value(|v| {
                                if v == *param {
                                    param_used_at_or_after = true;
                                }
                            });
                        }

                        if !param_used_at_or_after {
                            coalesced.insert(*arg, *param);
                        }
                    }
                    Value::Instruction { .. } | Value::Param { .. } => {
                        // Param-side: already-allocated value from another block or a block parameter.
                        // Safe when:
                        // 1. dest has exactly one predecessor (source = idom(dest)), ensuring
                        //    arg is allocated before param's definition point in convert_block_params.
                        //    (With multiple predecessors, idom(dest) may be an ancestor where arg
                        //    isn't allocated yet.)
                        // 2. arg is not live-in to dest (no interference in the destination block).
                        if cfg.predecessors(*destination).count() == 1
                            && !dest_live_in.contains(arg)
                        {
                            coalesced.insert(*param, *arg);
                        }
                    }
                    _ => {} // constants, globals — skip
                }
            }
        }

        Self { coalesced }
    }

    /// Look up whether `value_id` has been coalesced with a partner.
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        self.coalesced.get(value_id).copied()
    }

    /// Check whether `value_id` is a coalesced argument (i.e., shares a register with a parameter).
    pub(crate) fn is_coalesced(&self, value_id: &ValueId) -> bool {
        self.coalesced.contains_key(value_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    use super::CoalescingMap;

    /// Look up the coalescing map by examining the actual SSA structure,
    /// rather than guessing ValueIds (which the parser may renumber).
    fn get_jmp_coalescing(
        src: &str,
        block_idx: usize,
        arg_idx: usize,
    ) -> (CoalescingMap, ValueId, ValueId) {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let coalescing = CoalescingMap::from_function(func, &liveness);

        let blocks: Vec<_> = func.reachable_blocks().into_iter().collect();
        let block = &func.dfg[blocks[block_idx]];
        let term = block.terminator().unwrap();
        let (dest, args) = match term {
            crate::ssa::ir::instruction::TerminatorInstruction::Jmp {
                destination,
                arguments,
                ..
            } => (*destination, arguments),
            _ => panic!("Expected Jmp terminator"),
        };
        let arg = args[arg_idx];
        let param = func.dfg[dest].parameters()[arg_idx];
        (coalescing, arg, param)
    }

    #[test]
    fn coalesces_simple_block_parameter() {
        // The add result in b1 is passed to b3's parameter.
        // The parameter is NOT live-in to b1, so the add can write directly
        // to the parameter's register.
        // Field 42 in b2 is a constant — not coalesced.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            jmpif v0 then: b1, else: b2
          b1():
            v2 = add v1, Field 42
            jmp b3(v2)
          b2():
            jmp b3(Field 42)
          b3(v3: Field):
            return v3
        }
        ";
        // b1 is block index 1 (b0=0, b1=1, b2=2, b3=3), arg index 0
        let (coalescing, arg, param) = get_jmp_coalescing(src, 1, 0);
        assert_eq!(coalescing.get_coalesced(&arg), Some(param));

        // Check b2's jmp — constant arg should NOT be coalesced
        let (coalescing, arg, _param) = get_jmp_coalescing(src, 2, 0);
        assert!(!coalescing.is_coalesced(&arg));
    }

    #[test]
    fn does_not_coalesce_when_param_live_and_used() {
        // Loop: v1 is a block parameter of b1 and is used in the `add` that defines v3.
        // Since v1 is an operand of v3's defining instruction, coalescing v3 → v1 would
        // clobber v1 before the add reads it. Must reject.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, v0
            jmpif v2 then: b2, else: b3
          b2():
            v3 = add v1, u32 1
            jmp b1(v3)
          b3():
            return v1
        }
        ";
        // b2 is block index 2, arg index 0
        let (coalescing, arg, _param) = get_jmp_coalescing(src, 2, 0);
        assert!(!coalescing.is_coalesced(&arg));
    }

    #[test]
    fn does_not_coalesce_when_arg_live_across_loop() {
        // v2 is computed in b0 and passed as arg to b1's param v3.
        // v2 is also used in b2 (inside the loop body), so it is live-in to b1.
        // Since b1 has two predecessors (b0 and b2), the back-edge from b2
        // would overwrite v3's register with a different value, destroying v2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = add v0, v1
            jmp b1(v2)
          b1(v3: u32):
            v4 = lt v3, v0
            jmpif v4 then: b2, else: b3
          b2():
            v5 = add v3, v2
            jmp b1(v5)
          b3():
            return v3
        }
        ";
        // b0 is block index 0 (RPO), arg index 0 — v2 -> v3
        let (coalescing, arg, _param) = get_jmp_coalescing(src, 0, 0);
        assert!(
            !coalescing.is_coalesced(&arg),
            "v2 should not be coalesced with v3: v2 is live across the b1 loop"
        );
    }

    #[test]
    fn coalesces_non_live_param() {
        // The add result is passed to b1's parameter.
        // The parameter is NOT live-in to b0 (entry block has empty live-in),
        // so the add can safely write to the parameter's register.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1(v1)
          b1(v2: Field):
            return v2
        }
        ";
        // b0 is block index 0, arg index 0
        let (coalescing, arg, param) = get_jmp_coalescing(src, 0, 0);
        assert_eq!(coalescing.get_coalesced(&arg), Some(param));
    }

    #[test]
    fn coalesces_block_param_passthrough() {
        // b1 receives v2 and passes it straight through to b2's param v3.
        // v2 is not live-in to b2 (its only use is the jmp), so param-side
        // coalescing makes v3 reuse v2's register.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1(v1)
          b1(v2: Field):
            jmp b2(v2)
          b2(v3: Field):
            return v3
        }
        ";
        let (coalescing, _arg, param) = get_jmp_coalescing(src, 1, 0);
        assert!(coalescing.is_coalesced(&param));
    }

    #[test]
    fn coalesces_cross_block_instruction_result() {
        // v1 is an instruction result defined in b0. b1 passes v1 to b2's param v2.
        // b2 has a single predecessor (b1), so idom(b2) = b1 and v1 is guaranteed
        // allocated before v2's definition. v1 is not live-in to b2, so param-side
        // coalescing makes v2 reuse v1's register.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1()
          b1():
            jmp b2(v1)
          b2(v2: Field):
            return v2
        }
        ";
        // b1 is block index 1, arg 0: v1 (defined in b0, not b1) → v2
        let (coalescing, _arg, param) = get_jmp_coalescing(src, 1, 0);
        dbg!(param);
        assert!(coalescing.is_coalesced(&param));
    }

    #[test]
    fn does_not_coalesce_cross_block_multi_predecessor() {
        // v2 is defined in b0 and passed from b1 to b3's param v4.
        // b3 has two predecessors (b1 and b2), so idom(b3) = b0. Since v4 is
        // allocated at the beginning of b0 (before v2's instruction runs),
        // we cannot guarantee v2 is allocated at that point.
        //
        // This could be optimized in the future by checking that arg is live-in
        // to idom(dest) using the dominator tree, which would confirm the arg
        // is allocated before the param's definition point.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: Field):
            v2 = add v1, Field 1
            jmpif v0 then: b1, else: b2
          b1():
            jmp b3(v2)
          b2():
            v3 = mul v1, Field 2
            jmp b3(v3)
          b3(v4: Field):
            return v4
        }
        ";
        // b1, arg 0: v2 → v4. Not coalesced due to multi-predecessor.
        let (coalescing, arg, param) = get_jmp_coalescing(src, 1, 0);
        assert!(
            !coalescing.is_coalesced(&arg),
            "v2 is cross-block, falls to param-side which requires single-predecessor"
        );
        assert!(
            !coalescing.is_coalesced(&param),
            "param-side coalescing requires single-predecessor destination"
        );
        // b2, arg 0: v3 → v4. v3 is defined in b2 (source block) so this is arg-side.
        let (coalescing, arg, param) = get_jmp_coalescing(src, 2, 0);
        // Arg-side coalescing succeeds because v3 is not live-in to b3.
        assert!(
            coalescing.is_coalesced(&arg),
            "arg-side coalescing works even with multi-predecessor dest"
        );
        // Param v4 is not a coalescing key on the arg-side path.
        assert!(
            !coalescing.is_coalesced(&param),
            "param-side coalescing requires single-predecessor destination"
        );
    }
}
