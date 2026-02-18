//! Pre-codegen block parameter coalescing for Brillig.
//!
//! This module analyzes `Jmp` terminators to determine which arguments can share
//! a register with their destination block parameter. When an argument is coalesced
//! with a parameter, the instruction defining the argument writes directly to the
//! parameter's register, eliminating the mov at the jmp site.

use crate::ssa::ir::{
    cfg::ControlFlowGraph,
    function::Function,
    instruction::TerminatorInstruction,
    value::{Value, ValueId},
};

use rustc_hash::FxHashMap as HashMap;

use super::variable_liveness::VariableLiveness;

/// Maps SSA argument ValueIds to the block parameter ValueId whose register they should reuse.
#[derive(Default)]
pub(crate) struct CoalescingMap {
    coalesced: HashMap<ValueId, ValueId>,
}

impl CoalescingMap {
    /// Analyze all `Jmp` terminators in the function and build the coalescing map.
    ///
    /// For each `(argument, parameter)` pair at a jmp, we check whether the argument
    /// can safely write to the parameter's register. This is safe when the parameter
    /// is not live at the point where the argument is defined, or becomes dead before
    /// the defining instruction.
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

            for (arg, param) in arguments.iter().zip(params.iter()) {
                if arg == param {
                    continue;
                }

                // Only coalesce instruction results — coalescing works by intercepting code gen's
                // variable definition to reuse the parameter's register, which only fires
                // for instruction results defined in the same block as the jmp.
                let Value::Instruction { instruction: defining_inst, .. } = &func.dfg[*arg] else {
                    continue;
                };

                // If arg is already coalesced, skip (whether to the same or different parameter).
                if coalesced.contains_key(arg) {
                    continue;
                }

                // The defining instruction must be in this block so that `define_variable`
                // runs here (where we know the parameter is already allocated).
                let instructions = func.dfg[block_id].instructions();
                let Some(def_pos) =
                    instructions.iter().position(|inst_id| inst_id == defining_inst)
                else {
                    continue;
                };

                // If arg is live-in to the destination block and the destination has
                // other predecessors, we must not coalesce. When the destination is a
                // loop header (or merge point), other predecessors will write different
                // values to param's register on subsequent iterations, destroying arg's
                // value while it is still needed.
                if dest_live_in.contains(arg) && cfg.predecessors(*destination).count() > 1 {
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
                if !param_used_at_or_after && let Some(term) = func.dfg[block_id].terminator() {
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
        }

        Self { coalesced }
    }

    /// Look up whether `arg` has been coalesced with a block parameter.
    pub(crate) fn get_coalesced_param(&self, arg: &ValueId) -> Option<ValueId> {
        self.coalesced.get(arg).copied()
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
        assert_eq!(coalescing.get_coalesced_param(&arg), Some(param));

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
        assert_eq!(coalescing.get_coalesced_param(&arg), Some(param));
    }
}
