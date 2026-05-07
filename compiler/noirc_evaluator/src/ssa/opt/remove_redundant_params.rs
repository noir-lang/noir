//! Redundant block parameter elimination.
//!
//! After `mem2reg` promotes memory operations to block parameters, many of those
//! parameters become redundant: every predecessor passes the same value, or the
//! parameter is only fed by itself (loop back-edge). This pass detects and
//! removes such parameters using a fixed-point dataflow analysis, closely
//! following Cranelift's [`remove_constant_phis`] algorithm.
//!
//! [`remove_constant_phis`]: https://docs.wasmtime.dev/api/src/cranelift_codegen/remove_constant_phis.rs.html
//!
//! ## Algorithm (three phases)
//!
//! **Phase 1 — Summarize.**
//! For each non-entry block, record its parameters and the arguments each
//! predecessor passes via `Jmp` / `JmpIf` terminators.
//!
//! **Phase 2 — Dataflow.**
//! Iterate in reverse post-order until a fixed point. Each block parameter
//! carries an [`AbstractValue`]:
//!
//! | Variant    | Meaning                                  |
//! |------------|------------------------------------------|
//! | `None`     | No value observed yet (lattice bottom).   |
//! | `One(v)`   | Exactly one distinct value flows here.    |
//! | `Many`     | Multiple distinct values (lattice top).   |
//!
//! The join rule is: `None ⊔ x = x`, `One(v) ⊔ One(v) = One(v)`,
//! `One(v) ⊔ One(w) = Many`, `Many ⊔ _ = Many`.
//!
//! Self-references (loop back-edges passing a param to itself) and transitive
//! chains are handled implicitly by the lattice: a block parameter argument is
//! resolved to its current abstract value before joining, so self-references
//! contribute `x ⊔ x = x` (a no-op) and chains converge over iterations.
//!
//! **Phase 3 — Transform.**
//! For every parameter that resolved to `One(v)`, replace all uses of the
//! parameter with `v`, remove the parameter from the block, and strip the
//! corresponding argument from all predecessor terminators.
//!
//! The entry block is never modified (its parameters are the function ABI).
//!
//! ## Pipeline placement
//!
//! Chained after every `mem2reg` call in the pipeline. `mem2reg` is likely to generate
//! redundant parameters when promoting memory variables.  

use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::Function,
        instruction::TerminatorInstruction,
        post_order::PostOrder,
        value::{ValueId, ValueMapping},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`remove_redundant_params`][self] module for more information.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn remove_redundant_params(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.remove_redundant_params();
        }
        self
    }
}

/// Lattice element for the abstract value flowing to a block parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AbstractValue {
    /// No value observed yet (bottom of the lattice).
    None,
    /// Exactly one distinct value flows to this parameter.
    One(ValueId),
    /// Two or more distinct values flow here (top — not redundant).
    Many,
}

impl AbstractValue {
    /// Lattice join: combine two abstract values.
    fn join(self, other: AbstractValue) -> Self {
        match (self, other) {
            (AbstractValue::None, other) | (other, AbstractValue::None) => other,
            (AbstractValue::Many, _) | (_, AbstractValue::Many) => AbstractValue::Many,
            (AbstractValue::One(a), AbstractValue::One(b)) => {
                if a == b {
                    AbstractValue::One(a)
                } else {
                    AbstractValue::Many
                }
            }
        }
    }
}

/// Summary of a single non-entry block: its parameters and the arguments
/// each predecessor passes (one `Vec<ValueId>` per incoming edge).
struct BlockSummary {
    params: Vec<ValueId>,
    edges: Vec<Vec<ValueId>>,
}

impl Function {
    pub(crate) fn remove_redundant_params(&mut self) {
        let cfg = ControlFlowGraph::with_function(self);
        let rpo = PostOrder::with_cfg(&cfg).into_vec_reverse();

        let summaries = self.summarize_block_params(&cfg, &rpo);
        if summaries.is_empty() {
            return;
        }

        let lattice = Self::solve_dataflow(&summaries, &rpo);

        self.apply_transforms(&cfg, &rpo, &summaries, &lattice);
    }

    /// Phase 1: Summarize — collect block params and predecessor arguments.
    fn summarize_block_params(
        &self,
        cfg: &ControlFlowGraph,
        rpo: &[BasicBlockId],
    ) -> HashMap<BasicBlockId, BlockSummary> {
        let mut summaries: HashMap<BasicBlockId, BlockSummary> = HashMap::default();

        for &block in rpo {
            if block == self.entry_block() {
                continue;
            }
            let params = self.dfg.block_parameters(block).to_vec();
            if params.is_empty() {
                continue;
            }

            let mut edges = Vec::new();
            for pred in cfg.predecessors(block) {
                let terminator = self.dfg[pred].unwrap_terminator();
                collect_edges_for_target(terminator, block, &mut edges);
            }

            summaries.insert(block, BlockSummary { params, edges });
        }

        summaries
    }

    /// Phase 2: Dataflow — iterate in RPO until fixed point.
    fn solve_dataflow(
        summaries: &HashMap<BasicBlockId, BlockSummary>,
        rpo: &[BasicBlockId],
    ) -> HashMap<ValueId, AbstractValue> {
        let mut lattice: HashMap<ValueId, AbstractValue> = HashMap::default();

        for summary in summaries.values() {
            for &param in &summary.params {
                lattice.insert(param, AbstractValue::None);
            }
        }

        let mut changed = true;
        while changed {
            changed = false;

            for &block in rpo {
                let Some(summary) = summaries.get(&block) else {
                    continue;
                };

                for edge in &summary.edges {
                    for (&param, &arg) in summary.params.iter().zip_eq(edge) {
                        let current = lattice[&param];
                        if current == AbstractValue::Many {
                            continue;
                        }

                        // Resolve the argument: if it's a block parameter in the
                        // lattice, use its abstract value; otherwise it's a "Group B"
                        // value (instruction result / entry param / constant) → One(arg).
                        let arg_val = lattice.get(&arg).copied().unwrap_or(AbstractValue::One(arg));
                        let new_val = current.join(arg_val);

                        if new_val != current {
                            lattice.insert(param, new_val);
                            changed = true;
                        }
                    }
                }
            }
        }

        lattice
    }

    /// Phase 3: Transform — replace redundant params and prune them from blocks
    /// and predecessor terminators.
    fn apply_transforms(
        &mut self,
        cfg: &ControlFlowGraph,
        rpo: &[BasicBlockId],
        summaries: &HashMap<BasicBlockId, BlockSummary>,
        lattice: &HashMap<ValueId, AbstractValue>,
    ) {
        let mut mapping = ValueMapping::default();

        for summary in summaries.values() {
            for &param in &summary.params {
                if let AbstractValue::One(value) = lattice[&param] {
                    mapping.insert(param, value);
                }
            }
        }

        if mapping.is_empty() {
            return;
        }

        // Remove redundant parameters from blocks and predecessor terminators.
        for (&block, summary) in summaries {
            let keep_list: Vec<bool> = summary
                .params
                .iter()
                .map(|p| !matches!(lattice[p], AbstractValue::One(_)))
                .collect();

            // Skip blocks where nothing is removed.
            if keep_list.iter().all(|&k| k) {
                continue;
            }

            // Filter the block's parameter list.
            let mut keep_list_iter = keep_list.iter().copied();
            self.dfg[block].parameters_mut().retain(|_| keep_list_iter.next().unwrap());

            // Strip corresponding arguments from predecessor terminators.
            for pred in cfg.predecessors(block) {
                remove_terminator_args(&mut self.dfg[pred], block, &keep_list);
            }
        }

        // Apply value replacements across all reachable blocks.
        for &block in rpo {
            self.dfg.replace_values_in_block(block, &mapping);
        }
    }
}

/// Collect all edges from a terminator to `target`, appending to `edges`.
///
/// A `JmpIf` where both branches target the same block produces two edges.
fn collect_edges_for_target(
    terminator: &TerminatorInstruction,
    target: BasicBlockId,
    edges: &mut Vec<Vec<ValueId>>,
) {
    match terminator {
        TerminatorInstruction::Jmp { destination, arguments, .. } => {
            debug_assert_eq!(*destination, target);
            edges.push(arguments.clone());
        }
        TerminatorInstruction::JmpIf {
            then_destination,
            then_arguments,
            else_destination,
            else_arguments,
            ..
        } => {
            if *then_destination == target {
                edges.push(then_arguments.clone());
            }
            if *else_destination == target {
                edges.push(else_arguments.clone());
            }
        }
        _ => unreachable!("ICE: Return/Unreachable cannot be a predecessor"),
    }
}

/// Remove arguments at positions where `keep_list[i]` is false from the
/// terminator of `pred_block` that targets `target`.
fn remove_terminator_args(
    pred_block: &mut crate::ssa::ir::basic_block::BasicBlock,
    target: BasicBlockId,
    keep_list: &[bool],
) {
    let remove_args = |args: &mut Vec<ValueId>| {
        let mut keep_list_iter = keep_list.iter().copied();
        args.retain(|_| keep_list_iter.next().unwrap());
    };

    let terminator = pred_block.unwrap_terminator_mut();
    match terminator {
        TerminatorInstruction::Jmp { arguments, .. } => {
            remove_args(arguments);
        }
        TerminatorInstruction::JmpIf {
            then_destination,
            then_arguments,
            else_destination,
            else_arguments,
            ..
        } => {
            if *then_destination == target {
                remove_args(then_arguments);
            }
            if *else_destination == target {
                remove_args(else_arguments);
            }
        }
        _ => unreachable!("ICE: Return/Unreachable cannot be a predecessor"),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;
    use crate::ssa::Ssa;
    use crate::ssa::opt::assert_ssa_does_not_change;

    #[test]
    fn same_value_from_all_predecessors() {
        // Both branches of a diamond pass the same value — the parameter is redundant.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 7)
          b2():
            jmp b3(Field 7)
          b3(v1: Field):
            return v1
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            return Field 7
        }
        ");
    }

    #[test]
    fn different_values_kept() {
        // Predecessors pass different values — the parameter must stay.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 7)
          b2():
            jmp b3(Field 8)
          b3(v1: Field):
            return v1
        }";

        assert_ssa_does_not_change(src, Ssa::remove_redundant_params);
    }

    #[test]
    fn loop_back_edge_self_reference() {
        // The loop body passes the param back to itself. The only "real" value
        // comes from the preheader, so the parameter is redundant.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(u32 5)
          b1(v0: u32):
            v1 = lt u32 0, v0
            jmpif v1 then: b2(), else: b3()
          b2():
            jmp b1(v0)
          b3():
            return v0
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            jmp b1()
          b1():
            v2 = lt u32 0, u32 5
            jmpif v2 then: b2(), else: b3()
          b2():
            jmp b1()
          b3():
            return u32 5
        }
        ");
    }

    #[test]
    fn entry_block_not_modified() {
        // Entry block parameters are the function ABI and must never be removed.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            return v0
        }";

        assert_ssa_does_not_change(src, Ssa::remove_redundant_params);
    }

    #[test]
    fn mixed_redundant_and_non_redundant() {
        // Block b3 has two params: the first is redundant (both pass Field 1),
        // the second is not (Field 2 vs Field 3).
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 1, Field 2)
          b2():
            jmp b3(Field 1, Field 3)
          b3(v1: Field, v2: Field):
            v3 = add v1, v2
            return v3
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 2)
          b2():
            jmp b3(Field 3)
          b3(v1: Field):
            v5 = add Field 1, v1
            return v5
        }
        ");
    }

    #[test]
    fn transitive_resolution() {
        // b1 receives Field 5 from b0, then passes it to b2. b2's param should
        // resolve transitively to Field 5 through b1's param.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(Field 5)
          b1(v0: Field):
            jmp b2(v0)
          b2(v1: Field):
            return v1
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            jmp b1()
          b1():
            jmp b2()
          b2():
            return Field 5
        }
        ");
    }

    #[test]
    fn jmpif_same_target_both_branches() {
        // Both branches of a jmpif target the same block with the same value.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(Field 42), else: b1(Field 42)
          b1(v1: Field):
            return v1
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b1()
          b1():
            return Field 42
        }
        ");
    }

    #[test]
    fn jmpif_same_target_different_values() {
        // Both branches target the same block but with different values — keep.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(Field 1), else: b1(Field 2)
          b1(v1: Field):
            return v1
        }";

        assert_ssa_does_not_change(src, Ssa::remove_redundant_params);
    }

    #[test]
    fn redundant_loop_carried_params() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0, v0, u32 0)
          b1(v2: u32, v3: u32, v4: u32):
            v5 = lt v2, u32 5
            jmpif v5 then: b2(v3, v4), else: b3(v4)
          b2(v6: u32, v7: u32):
            v8 = add v7, u32 10
            v9 = unchecked_add v2, u32 1
            jmp b1(v9, v6, v8)
          b3(v10: u32):
            return v10
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_params();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0, u32 0)
          b1(v1: u32, v2: u32):
            v7 = lt v1, u32 5
            jmpif v7 then: b2(v2), else: b3(v2)
          b2(v3: u32):
            v9 = add v3, u32 10
            v11 = unchecked_add v1, u32 1
            jmp b1(v11, v9)
          b3(v4: u32):
            return v4
        }
        ");
    }
}
