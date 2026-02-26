//! Pre-codegen block parameter coalescing for Brillig.
//!
//! This module analyzes `Jmp` terminators to determine which arguments can share
//! a register with their destination block parameter. When an argument is coalesced
//! with a parameter, the instruction defining the argument writes directly to the
//! parameter's register, eliminating the mov at the jmp site.

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    cfg::ControlFlowGraph,
    function::Function,
    instruction::TerminatorInstruction,
    value::{Value, ValueId},
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::variable_liveness::VariableLiveness;

/// Check if param-side coalescing is safe: the destination must have exactly
/// one predecessor, and arg must not be live-in to dest.
fn can_coalesce_param_side(
    arg: &ValueId,
    destination: &BasicBlockId,
    dest_live_in: &HashSet<ValueId>,
    cfg: &ControlFlowGraph,
) -> bool {
    cfg.predecessors(*destination).count() == 1 && !dest_live_in.contains(arg)
}

/// Maps SSA ValueIds to partners whose register they should reuse.
///
/// An entry `x -> y` means "when defining x, reuse y's register."
/// This covers two cases:
/// - Arg-side (`arg -> param`): an instruction result defined in the same block as the jmp
///   writes directly to the parameter's register.
/// - Param-side (`param -> arg`): a block parameter reuses the register of an already-allocated
///   value (block param passthrough, cross-block instruction result).
#[derive(Default)]
pub(crate) struct CoalescingMap {
    coalesced: HashMap<ValueId, ValueId>,
    /// Reverse mapping: values that are targets of coalescing.
    /// Maps a coalescing target back to its source.
    coalesced_reverse: HashMap<ValueId, ValueId>,
}

impl CoalescingMap {
    /// Analyze all `Jmp` terminators in the function and build the coalescing map.
    ///
    /// For each `(argument, parameter)` pair at a jmp, we attempt to coalesce in one
    /// of two directions:
    /// - Arg-side: if the arg is an instruction result defined in the source block,
    ///   record `arg -> param` so the instruction writes to the param's register.
    /// - Param-side: if the arg is a block parameter or instruction from another block,
    ///   record `param -> arg` so the param reuses the arg's register.
    pub(crate) fn from_function(func: &Function, liveness: &VariableLiveness) -> Self {
        let mut coalesced = HashMap::default();
        let cfg = liveness.cfg();

        for block_id in func.reachable_blocks() {
            let block = &func.dfg[block_id];

            let Some(TerminatorInstruction::Jmp { destination, arguments, .. }) =
                block.terminator()
            else {
                continue;
            };

            let dest_block = &func.dfg[*destination];
            let params = dest_block.parameters();
            let dest_live_in = liveness.get_live_in(destination);
            let instructions = func.dfg[block_id].instructions();

            // Track values used as param-side targets within this jmp to prevent
            // two params of the same destination from reusing the same register.
            let mut param_side_targets = HashSet::default();

            for (arg, param) in arguments.iter().zip(params.iter()) {
                if arg == param {
                    continue;
                }

                // Skip if this arg or param is already committed to a coalescing.
                // - Arg-side guard: the same arg must not map to multiple params across jmps.
                // - Param-side guard: two params in the same jmp must not reuse the same arg's register.
                if coalesced.contains_key(arg) || param_side_targets.contains(arg) {
                    continue;
                }

                // Globals are allocated separately (in the globals map, not ssa_value_allocations),
                // so they cannot participate in coalescing. Note: we must check via `is_global`
                // rather than matching on `func.dfg[*arg]` because the DFG's Index impl
                // transparently resolves Global values to their underlying instruction in the
                // globals graph, hiding the Global wrapper.
                if func.dfg.is_global(*arg) {
                    continue;
                }

                match &func.dfg[*arg] {
                    Value::Instruction { instruction: defining_inst, .. } => {
                        if let Some(def_pos) =
                            instructions.iter().position(|id| id == defining_inst)
                        {
                            // Arg-side: instruction defined in this block.

                            // If arg is live-in to the destination block and the destination has
                            // other predecessors, we must not coalesce. When the destination is a
                            // loop header (or merge point), other predecessors will write different
                            // values to param's register on subsequent iterations, destroying arg's
                            // value while it is still needed.
                            if dest_live_in.contains(arg)
                                && cfg.predecessors(*destination).count() > 1
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

                            // Check if param is used in the defining instruction or any subsequent
                            // instruction, or in the block terminator.
                            let param_used_at_or_after = instructions[def_pos..]
                                .iter()
                                .any(|inst_id| func.dfg[*inst_id].any_value(|v| v == *param))
                                || func.dfg[block_id]
                                    .unwrap_terminator()
                                    .any_value(|v| v == *param);

                            if !param_used_at_or_after {
                                coalesced.insert(*arg, *param);
                            }
                        } else if can_coalesce_param_side(arg, destination, dest_live_in, cfg) {
                            // Param-side: instruction from another block.
                            coalesced.insert(*param, *arg);
                            param_side_targets.insert(*arg);
                        }
                    }
                    Value::Param { .. } => {
                        // Param-side: block parameter passthrough.
                        if can_coalesce_param_side(arg, destination, dest_live_in, cfg) {
                            coalesced.insert(*param, *arg);
                            param_side_targets.insert(*arg);
                        }
                    }
                    _ => {} // constants, globals — skip
                }
            }
        }

        let coalesced_reverse = coalesced.iter().map(|(k, v)| (*v, *k)).collect::<HashMap<_, _>>();
        Self { coalesced, coalesced_reverse }
    }

    /// Forward-only lookup: if `value_id` is a coalesced arg, returns the param
    /// whose register it reuses. Used when defining a variable to check whether
    /// it should share an already-allocated param's register.
    pub(crate) fn get_coalesced(&self, value_id: &ValueId) -> Option<ValueId> {
        self.coalesced.get(value_id).copied()
    }

    /// Check whether `value_id` is a coalesced argument (i.e., shares a register with a parameter).
    #[cfg(test)]
    pub(crate) fn is_coalesced(&self, value_id: &ValueId) -> bool {
        self.coalesced.contains_key(value_id)
    }

    /// Bidirectional lookup: returns the coalescing partner of `value_id`,
    /// regardless of whether it is the arg or the param in the pair. Used when
    /// a value dies to check whether its partner is still alive and sharing
    /// the same register.
    pub(crate) fn get_partner(&self, value_id: &ValueId) -> Option<ValueId> {
        self.coalesced.get(value_id).or_else(|| self.coalesced_reverse.get(value_id)).copied()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.coalesced.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::brillig::brillig_gen::constant_allocation::ConstantAllocation;
    use crate::brillig::brillig_gen::variable_liveness::VariableLiveness;
    use crate::ssa::ir::value::ValueId;
    use crate::ssa::ssa_gen::Ssa;

    use super::CoalescingMap;

    /// Parse SSA source and build the coalescing map once.
    fn build_coalescing(src: &str) -> (CoalescingMap, Ssa) {
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let constants = ConstantAllocation::from_function(func);
        let liveness = VariableLiveness::from_function(func, &constants);
        let coalescing = CoalescingMap::from_function(func, &liveness);
        (coalescing, ssa)
    }

    /// Look up the (arg, param) pair for a specific jmp terminator.
    fn get_jmp_pair(
        func: &crate::ssa::ir::function::Function,
        block_idx: usize,
        arg_idx: usize,
    ) -> (ValueId, ValueId) {
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
        (arg, param)
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
            jmpif v0 then: b1(), else: b2()
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
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert_eq!(coalescing.get_coalesced(&arg), Some(param));

        // Check b2's jmp — constant arg should NOT be coalesced
        let (arg, param) = get_jmp_pair(func, 2, 0);
        assert!(!coalescing.is_coalesced(&arg));
        assert!(!coalescing.is_coalesced(&param));

        assert_eq!(coalescing.len(), 1);
    }

    #[test]
    fn does_not_coalesce_when_param_live_and_used() {
        // Loop: v1 is a block parameter of b1 and is used in the `add` that defines v3.
        // Since v1 is an operand of v3's defining instruction, coalescing v3 -> v1 would
        // clobber v1 before the add reads it. Must reject.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, v0
            jmpif v2 then: b2(), else: b3()
          b2():
            v3 = add v1, u32 1
            jmp b1(v3)
          b3():
            return v1
        }
        ";
        // b2 is block index 2, arg index 0
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 2, 0);
        assert!(!coalescing.is_coalesced(&arg));
        assert!(!coalescing.is_coalesced(&param));
        assert_eq!(coalescing.len(), 0);
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
            jmpif v4 then: b2(), else: b3()
          b2():
            v5 = add v3, v2
            jmp b1(v5)
          b3():
            return v3
        }
        ";
        // b0 is block index 0 (RPO), arg index 0 — v2 -> v3
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 0, 0);
        assert!(
            !coalescing.is_coalesced(&arg),
            "v2 should not be coalesced with v3: v2 is live across the b1 loop"
        );
        assert!(!coalescing.is_coalesced(&param));
        assert_eq!(coalescing.len(), 0);
    }

    #[test]
    fn does_not_coalesce_when_arg_equals_param() {
        // Loop back-edge where b2 passes v1 (b1's own param) back to b1.
        // Since arg == param (same value), we should skip the pair.
        // b0->b1 passes v0 to v1, but b1 has 2 predecessors and v0 is a
        // function param, so v0 -> v1 is rejected. Net result: no coalescing.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(v0)
          b1(v1: u32):
            v2 = lt v1, u32 10
            jmpif v2 then: b2(), else: b3()
          b2():
            jmp b1(v1)
          b3():
            return v1
        }
        ";
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        // b2 is block index 2, arg index 0: v1 -> v1
        let (arg, param) = get_jmp_pair(func, 2, 0);
        assert_eq!(arg, param, "arg and param should be the same ValueId");
        assert!(!coalescing.is_coalesced(&arg), "arg == param should be skipped");
        assert_eq!(coalescing.len(), 0);
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
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();

        // b0->b1: v1 is an instruction result in b0, arg-side coalescing v1->v2
        let (arg, param) = get_jmp_pair(func, 0, 0);
        assert_eq!(coalescing.get_coalesced(&arg), Some(param));

        // b1->b2: v2 is a block param passed through, param-side coalescing v3->v2
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert_eq!(coalescing.get_coalesced(&param), Some(arg));

        assert_eq!(coalescing.len(), 2);
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
        // b1 is block index 1, arg 0: v1 (defined in b0, not b1) -> v2
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert_eq!(coalescing.get_coalesced(&param), Some(arg));
        assert_eq!(coalescing.len(), 1);
    }

    #[test]
    fn does_not_coalesce_global_arg() {
        // g0 is a global array passed as arg to b2's param v1.
        // Even though b2 has a single predecessor (b1) and g0 is not live-in to b2,
        // globals are allocated separately (in the globals map, not ssa_value_allocations),
        // so param-side coalescing must be skipped.
        //
        // The DFG's Index<ValueId> transparently resolves Value::Global to its
        // underlying instruction in the globals graph, so the coalescing code must
        // check `dfg.is_global()` rather than matching on the resolved value.
        let src = "
        g0 = make_array [u8 65] : [u8; 1]

        brillig(inline) fn main f0 {
          b0(v0: [u8; 1]):
            v1 = call f1() -> u1
            jmpif v1 then: b1(), else: b2()
          b1():
            inc_rc g0
            jmp b3(g0)
          b2():
            constrain u1 0 == u1 1
            unreachable
          b3(v2: [u8; 1]):
            return v2
        }
        brillig(inline) fn func_3 f1 {
          b0():
            return u1 0
        }
        ";
        // b1 is block index 1, arg 0: g0 -> v2
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert!(!coalescing.is_coalesced(&arg), "global arg should not be coalesced");
        assert!(!coalescing.is_coalesced(&param), "param should not be coalesced with global arg");
        assert_eq!(coalescing.len(), 0);
    }

    #[test]
    fn coalesces_when_param_live_but_last_use_before_def() {
        // Loop where v1 (param of b1, the destination) is live-in to b2 because
        // it's used in the add. But v4's defining instruction (mul) comes AFTER
        // v1's last use, so param_used_at_or_after is false and coalescing succeeds.
        //
        // Param IS live-in to the source block, but is not used at or after the arg's defining instruction.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, v0
            jmpif v2 then: b2(), else: b3()
          b2():
            v3 = add v1, u32 1
            v4 = mul v3, u32 2
            jmp b1(v4)
          b3():
            return v1
        }
        ";
        // b2 is block index 2, arg index 0: v4 -> v1
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 2, 0);
        assert_eq!(
            coalescing.get_coalesced(&arg),
            Some(param),
            "v4 should coalesce with v1: v1's last use is before v4's defining instruction"
        );
        assert_eq!(coalescing.len(), 1);
    }

    #[test]
    fn does_not_coalesce_when_param_used_in_terminator() {
        // Loop with two block parameters. In b2, v4 is defined (mul) without using
        // v1 in its defining instruction. However, v1 appears as the SECOND argument
        // in the jmp terminator. The terminator check must catch this: coalescing
        // v4 -> v1 would clobber v1's register before the jmp reads it as the second arg.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0, u32 10)
          b1(v1: u32, v2: u32):
            v3 = lt v1, v0
            jmpif v3 then: b2(), else: b3()
          b2():
            v4 = mul v0, u32 2
            jmp b1(v4, v1)
          b3():
            return v2
        }
        ";
        // b2 is block index 2, arg index 0: v4 -> v1
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 2, 0);
        assert!(
            !coalescing.is_coalesced(&arg),
            "v4 must not coalesce with v1: v1 is used in the terminator as the second arg"
        );
        assert!(!coalescing.is_coalesced(&param));
        assert_eq!(coalescing.len(), 0);
    }

    #[test]
    fn does_not_coalesce_param_side_when_arg_live_in_dest() {
        // v1 is defined in b0 and passed from b1 to b2's param v2.
        // b2 has a single predecessor (b1), satisfying the first param-side condition.
        // However, v1 is also used directly in b2, making it live-in to b2.
        // Coalescing v2 -> v1 would create interference: both v1 and v2 are live
        // simultaneously in b2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1()
          b1():
            jmp b2(v1)
          b2(v2: Field):
            v3 = add v1, v2
            return v3
        }
        ";
        // b1 is block index 1, arg 0: v1 -> v2
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert!(!coalescing.is_coalesced(&arg));
        assert!(!coalescing.is_coalesced(&param), "param-side must reject: v1 is live-in to b2");
        assert_eq!(coalescing.len(), 0);
    }

    #[test]
    fn does_not_double_coalesce_same_arg() {
        // The same value v1 is passed as both arguments to b1's two parameters.
        // The first pair (v1 -> v2) coalesces, but the second pair (v1 -> v3) must be
        // skipped because v1 is already a key in the coalescing map.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1(v1, v1)
          b1(v2: Field, v3: Field):
            v4 = add v2, v3
            return v4
        }
        ";
        // b0 is block index 0, check both arg indices
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg0, param0) = get_jmp_pair(func, 0, 0);
        let (arg1, param1) = get_jmp_pair(func, 0, 1);

        // arg0 and arg1 are the same ValueId (v1)
        assert_eq!(arg0, arg1, "both args should be the same value");

        // Exactly one of the two pairs should coalesce (the first one encountered)
        let first_coalesced = coalescing.is_coalesced(&arg0);
        let second_param_coalesced = coalescing.is_coalesced(&param1);

        assert!(first_coalesced, "first pair (v1 -> v2) should coalesce");
        assert_eq!(
            coalescing.get_coalesced(&arg0),
            Some(param0),
            "v1 should map to v2 (the first param)"
        );
        assert!(
            !second_param_coalesced,
            "second param v3 must not coalesce: v1 is already a key from the first pair"
        );
        assert_eq!(coalescing.len(), 1);
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
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(v2)
          b2():
            v3 = mul v1, Field 2
            jmp b3(v3)
          b3(v4: Field):
            return v4
        }
        ";
        // b1, arg 0: v2 -> v4. Not coalesced due to multi-predecessor.
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (arg, param) = get_jmp_pair(func, 1, 0);
        assert!(
            !coalescing.is_coalesced(&arg),
            "v2 is cross-block, falls to param-side which requires single-predecessor"
        );
        assert!(
            !coalescing.is_coalesced(&param),
            "param-side coalescing requires single-predecessor destination"
        );
        // b2, arg 0: v3 -> v4. v3 is defined in b2 (source block) so this is arg-side.
        let (arg, param) = get_jmp_pair(func, 2, 0);
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
        assert_eq!(coalescing.len(), 1);
    }

    #[test]
    fn does_not_coalesce_same_arg_to_two_params_param_side() {
        // v1 is a cross-block instruction result passed as both arguments to b2.
        // The first pair (v2 -> v1) coalesces param-side, but the second pair (v3 -> v1)
        // must be skipped because v1 is already a param-side target within this jmp.
        // Without this check, both v2 and v3 would try to reuse v1's register.
        // If v2 died early and its register got reclaimed while v3 was still live
        // we would get corrupted values.
        // In this specific case though even though the liveness intervals of v2 and v3 overlap
        // they are safe to coalesce. This type of analysis will be easier once we add liveness intervals.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 1
            jmp b1()
          b1():
            jmp b2(v1, v1)
          b2(v2: Field, v3: Field):
            v4 = add v2, v3
            return v4
        }
        ";
        // b1 is block index 1, both arg indices
        let (coalescing, ssa) = build_coalescing(src);
        let func = ssa.main();
        let (_arg0, param0) = get_jmp_pair(func, 1, 0);
        let (_arg1, param1) = get_jmp_pair(func, 1, 1);

        // Exactly one of the two params should coalesce with v1
        let first_coalesced = coalescing.is_coalesced(&param0);
        let second_coalesced = coalescing.is_coalesced(&param1);
        assert!(first_coalesced ^ second_coalesced, "exactly one param should coalesce with v1");
        // The first pair encountered wins
        assert!(first_coalesced, "first param should coalesce");
        assert!(
            !second_coalesced,
            "second param must not coalesce: v1 is already a param-side target"
        );
    }
}
