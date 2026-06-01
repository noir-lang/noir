//! # Brillig Function Specialization
//!
//! ## Problem
//!
//! When a Brillig function is called with constant arguments but is too large to inline,
//! those constants don't propagate into the function body. For example, `main` calls
//! `foo(1, x)` — the body of `foo` sees generic parameters `v0, v1` and can't fold
//! `eq v0, 1` into `true`, leaving a dead branch in the compiled Brillig bytecode.
//!
//! ## Solution
//!
//! This pass creates specialized clones with constants substituted, runs optimization
//! passes on each clone, and keeps only those that shrink enough to justify the code-size
//! cost. Continuing the example above: we clone `foo`, replace `v0` with `1`, run constant
//! folding — `eq v0, 1` folds to `true`, the false branch is eliminated, and the clone is
//! measurably smaller. If savings exceed the threshold, the clone is kept.
//!
//! ## Process
//!
//! 1. **Scan** — find candidate call sites: Brillig callee, not recursive, at least one
//!    numeric constant argument.
//! 2. **Clone & measure** — for each candidate, clone the callee, substitute constants,
//!    run optimization passes, and compare instruction counts. Keep only clones whose
//!    savings exceed the threshold (capped at N clones per original function).
//! 3. **Rewrite callers** — rewrite call sites in the original callers to point at the
//!    specialized clones.
//! 4. **Rewrite clones** — rewrite call sites *inside* surviving clones that match other
//!    specializations (nested specialization).
//!
//! ## Post-conditions
//!
//! Specialized clones are added to the SSA. Original functions are left untouched — dead
//! function elimination cleans them up later if all call sites were rewritten.
//!
//! This is Brillig-only — ACIR functions get fully inlined anyway.

use std::collections::BTreeMap;

use acvm::FieldElement;
use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        call_graph::CallGraph,
        function::{Function, FunctionId},
        instruction::{Instruction, InstructionId},
        types::NumericType,
        value::{Value, ValueId, ValueMapping},
    },
    ssa_gen::Ssa,
};

/// Minimum percentage cost reduction required to keep a specialized clone.
pub const DEFAULT_SPECIALIZATION_THRESHOLD: usize = 20;

/// Maximum number of specialized clones per original function.
pub const DEFAULT_MAX_SPECIALIZATIONS_PER_FN: usize = 3;

/// A specialization key: the callee function ID and the constant pattern per parameter position.
/// `None` means the argument is not a constant at that call site.
///
/// Only numeric constants are tracked. Array arguments (even fully constant `MakeArray`s) are
/// intentionally excluded: substituting a constant array into the specialized clone creates a
/// fresh `MakeArray` with an independent reference count. The caller's `inc_rc` still targets
/// the original array, so the clone's copy starts at RC 1 instead of the expected value. This
/// breaks `array_refcount` assertions and could silently alter copy-on-write behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpecializationKey {
    callee: FunctionId,
    constants: Vec<Option<(FieldElement, NumericType)>>,
}

/// A candidate call site that matches a specialization key.
struct CallSite {
    /// The function containing this call instruction.
    caller: FunctionId,
    /// The instruction ID of the call.
    instruction_id: InstructionId,
}

impl Ssa {
    /// Specialize Brillig functions called with constant arguments.
    ///
    /// Setting `specialization_threshold` to 0 disables the pass.
    pub(crate) fn brillig_function_specialization(
        mut self,
        specialization_threshold: usize,
        max_specializations_per_fn: usize,
        constant_folding_max_iter: usize,
    ) -> Ssa {
        if specialization_threshold == 0 {
            return self;
        }

        // Phase 1: Scan for candidates
        let call_graph = CallGraph::from_ssa(&self);
        let recursive_functions = call_graph.get_recursive_functions();

        // Collect all call sites grouped by specialization key.
        let mut key_to_callsites = collect_specialization_candidates(&self, &recursive_functions);

        if key_to_callsites.is_empty() {
            return self;
        }

        // Phase 2: Clone candidates, optimize, keep survivors.
        let surviving = create_specialized_clones(
            &mut self,
            &key_to_callsites,
            specialization_threshold,
            max_specializations_per_fn,
            constant_folding_max_iter,
        );
        if surviving.is_empty() {
            return self;
        }

        // Phase 3: Extend key_to_callsites with call sites inside surviving clones
        // that match other specializations (nested specialization).
        let clone_ids: Vec<FunctionId> = surviving.values().copied().collect();
        collect_matching_callsites(&self.functions, &clone_ids, &surviving, &mut key_to_callsites);

        // Phase 4: Rewrite all call sites (original callers + clones) in one pass.
        rewrite_call_sites(&mut self.functions, &key_to_callsites, &surviving);

        self
    }
}

/// Phase 1: Scan all functions for call sites where:
/// - Callee is a Brillig function (not ACIR, not intrinsic/foreign)
/// - Callee is NOT recursive
/// - At least one argument is a numeric constant
///
/// Array arguments are not considered constant even if all elements are constant, because
/// substituting them would create a new array object with independent reference counting.
///
/// Returns an `IndexMap` from specialization key to call sites, preserving insertion order
/// (first occurrence) for deterministic iteration.
fn collect_specialization_candidates(
    ssa: &Ssa,
    recursive_functions: &HashSet<FunctionId>,
) -> IndexMap<SpecializationKey, Vec<CallSite>> {
    let mut key_to_callsites: IndexMap<SpecializationKey, Vec<CallSite>> = IndexMap::new();

    for (caller_id, caller_fn) in &ssa.functions {
        for block_id in caller_fn.reachable_blocks() {
            for instruction_id in caller_fn.dfg[block_id].instructions() {
                let instruction = &caller_fn.dfg[*instruction_id];
                let Instruction::Call { func: func_value_id, arguments } = instruction else {
                    continue;
                };

                let Value::Function(callee_id) = &caller_fn.dfg[*func_value_id] else {
                    continue;
                };

                // Must be a Brillig function that exists in the SSA.
                let Some(callee_fn) = ssa.functions.get(callee_id) else {
                    continue;
                };
                if !callee_fn.runtime().is_brillig() {
                    continue;
                }

                // Skip recursive functions.
                if recursive_functions.contains(callee_id) {
                    continue;
                }

                let Some(key) = try_build_specialization_key(caller_fn, *callee_id, arguments)
                else {
                    continue;
                };

                key_to_callsites
                    .entry(key)
                    .or_default()
                    .push(CallSite { caller: *caller_id, instruction_id: *instruction_id });
            }
        }
    }

    // Sort by the number of call sites (descending) so we specialize the most frequently
    // occurring keys before running into the per-callee limit.
    key_to_callsites
        .sort_by(|_, call_sites_a, _, call_sites_b| call_sites_b.len().cmp(&call_sites_a.len()));

    key_to_callsites
}

/// Try to build a [SpecializationKey] for a call instruction.
/// Returns `None` if no argument is a numeric constant.
fn try_build_specialization_key(
    caller_fn: &Function,
    callee_id: FunctionId,
    arguments: &[ValueId],
) -> Option<SpecializationKey> {
    let constants: Vec<Option<(FieldElement, NumericType)>> =
        arguments.iter().map(|arg| caller_fn.dfg.get_numeric_constant_with_type(*arg)).collect();

    if !constants.iter().any(|c| c.is_some()) {
        return None;
    }

    Some(SpecializationKey { callee: callee_id, constants })
}

/// Substitute constant values into the clone's entry block parameters.
/// For each parameter position that has a constant in the key, create
/// that constant in the clone's DFG and set up a value mapping.
fn substitute_constants(
    function: &mut Function,
    constants: &[Option<(FieldElement, NumericType)>],
) {
    let entry_block = function.entry_block();
    let params: Vec<ValueId> = function.dfg.block_parameters(entry_block).to_vec();

    let mut mapping = ValueMapping::default();
    for (param, constant) in params.iter().zip_eq(constants.iter()) {
        if let Some((field, typ)) = constant {
            let const_value = function.dfg.make_constant(*field, *typ);
            mapping.insert(*param, const_value);
        }
    }

    // Apply the mapping to all reachable blocks.
    for block_id in function.reachable_blocks() {
        function.dfg.replace_values_in_block(block_id, &mapping);
    }
}

/// Run lightweight per-function optimization passes on the clone.
fn optimize_clone(
    function: &mut Function,
    constant_folding_max_iter: usize,
    all_functions: &BTreeMap<FunctionId, Function>,
) {
    use crate::ssa::interpreter::{Interpreter, InterpreterOptions};

    // Simplify CFG: fold constant jmpifs, merge blocks.
    function.simplify_function();

    // Constant folding: propagate the substituted constants.
    let brillig_functions: BTreeMap<FunctionId, Function> = all_functions
        .iter()
        .filter(|(_, func)| func.runtime().is_brillig())
        .map(|(id, func)| (*id, func.clone()))
        .collect();

    let mut interpreter = Interpreter::new_from_functions(
        &brillig_functions,
        InterpreterOptions {
            no_foreign_calls: true,
            step_limit: Some(10_000_000),
            ..Default::default()
        },
        std::io::empty(),
    );
    interpreter.interpret_globals().expect("ICE: Interpreter failed to interpret globals");

    function.constant_fold(false, constant_folding_max_iter, &mut interpreter);

    // Simplify CFG again after folding to merge any newly-dead blocks.
    // Note: We skip per-function DIE here because it is intentionally private
    // (it can leave dangling block parameters when run in isolation).
    // The full-SSA DIE pass will clean up dead code in surviving clones later.
    function.simplify_function();
}

/// For each candidate key, clone the callee, substitute constants, run optimization passes,
/// and keep the clone only if savings exceed the threshold. Returns a map from surviving
/// keys to their new function IDs.
fn create_specialized_clones(
    ssa: &mut Ssa,
    key_to_callsites: &IndexMap<SpecializationKey, Vec<CallSite>>,
    specialization_threshold: usize,
    max_specializations_per_fn: usize,
    constant_folding_max_iter: usize,
) -> HashMap<SpecializationKey, FunctionId> {
    // Group keys by callee to enforce the per-function cap.
    let mut keys_per_callee: BTreeMap<FunctionId, Vec<SpecializationKey>> = BTreeMap::new();
    for key in key_to_callsites.keys() {
        keys_per_callee.entry(key.callee).or_default().push(key.clone());
    }

    let mut surviving: HashMap<SpecializationKey, FunctionId> = HashMap::default();

    for keys in keys_per_callee.values() {
        let mut specializations_for_callee = 0;

        let callee_id = keys[0].callee;
        let original_cost = ssa.functions[&callee_id].cost();
        if original_cost == 0 {
            continue;
        }

        for key in keys {
            if specializations_for_callee >= max_specializations_per_fn {
                break;
            }

            // Clone and substitute constants.
            let mut clone = ssa.functions[&callee_id].clone();
            substitute_constants(&mut clone, &key.constants);

            // Run per-function optimization passes on the clone.
            optimize_clone(&mut clone, constant_folding_max_iter, &ssa.functions);

            let specialized_cost = clone.cost();
            let savings_percent = if original_cost > specialized_cost {
                ((original_cost - specialized_cost) * 100) / original_cost
            } else {
                0
            };

            if savings_percent >= specialization_threshold {
                let new_id = ssa.add_fn(|id| Function::clone_with_id(id, &clone));
                surviving.insert(key.clone(), new_id);
                specializations_for_callee += 1;
            }
        }
    }

    surviving
}

/// Rewrite call instructions in original callers to point at the specialized clone
/// instead of the original callee.
fn rewrite_call_sites(
    functions: &mut BTreeMap<FunctionId, Function>,
    key_to_callsites: &IndexMap<SpecializationKey, Vec<CallSite>>,
    surviving: &HashMap<SpecializationKey, FunctionId>,
) {
    for (key, new_fn_id) in surviving {
        let callsites = &key_to_callsites[key];
        for callsite in callsites {
            let caller_fn =
                functions.get_mut(&callsite.caller).expect("ICE: caller function not found");
            let new_func_value = caller_fn.dfg.import_function(*new_fn_id);

            let instruction = &mut caller_fn.dfg[callsite.instruction_id];
            let Instruction::Call { func, .. } = instruction else {
                unreachable!("ICE: expected Call instruction");
            };
            *func = new_func_value;
        }
    }
}

/// Scan the given functions for call sites that match a surviving specialization key
/// and append them to `key_to_callsites`. This lets [rewrite_call_sites] handle both
/// original callers and clones in one pass.
fn collect_matching_callsites(
    functions: &BTreeMap<FunctionId, Function>,
    function_ids: &[FunctionId],
    surviving: &HashMap<SpecializationKey, FunctionId>,
    key_to_callsites: &mut IndexMap<SpecializationKey, Vec<CallSite>>,
) {
    for &fn_id in function_ids {
        let function = &functions[&fn_id];
        for block_id in function.reachable_blocks() {
            for instruction_id in function.dfg[block_id].instructions() {
                let Instruction::Call { func: func_value_id, arguments } =
                    &function.dfg[*instruction_id]
                else {
                    continue;
                };
                let Value::Function(callee_id) = &function.dfg[*func_value_id] else {
                    continue;
                };

                if let Some(key) = try_build_specialization_key(function, *callee_id, arguments)
                    && surviving.contains_key(&key)
                {
                    key_to_callsites
                        .entry(key)
                        .or_default()
                        .push(CallSite { caller: fn_id, instruction_id: *instruction_id });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;
    use crate::ssa::{
        ir::{instruction::Instruction, value::Value},
        ssa_gen::Ssa,
    };

    use super::{DEFAULT_MAX_SPECIALIZATIONS_PER_FN, DEFAULT_SPECIALIZATION_THRESHOLD};

    fn run_specialization(src: &str) -> Ssa {
        run_specialization_with_options(
            src,
            DEFAULT_SPECIALIZATION_THRESHOLD,
            DEFAULT_MAX_SPECIALIZATIONS_PER_FN,
        )
    }

    fn run_specialization_with_options(src: &str, threshold: usize, max_per_fn: usize) -> Ssa {
        let ssa = Ssa::from_str(src).unwrap();
        ssa.brillig_function_specialization(threshold, max_per_fn, 5)
    }

    #[test]
    fn basic_constant_specialization() {
        // A Brillig function called with a constant argument should be specialized.
        // The constant `u32 1` is propagated, allowing `eq v0, u32 1` to simplify.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 1, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, Field 1
            jmp b3(v3)
          b2():
            v4 = sub v1, Field 1
            jmp b3(v4)
          b3(v5: Field):
            constrain v5 == Field 0
            return
        }
        ";

        let ssa = run_specialization(src);

        // The specialized clone should have the constant branch folded away.
        // Original f1 should still exist, but the call site in main should point to the new clone.
        // The new clone should be simpler (one branch eliminated).
        let main_fn = ssa.main();
        let entry_block = main_fn.entry_block();
        let instructions = main_fn.dfg[entry_block].instructions();

        // Find the call instruction and verify it calls a different function than f1.
        let mut found_specialized_call = false;
        for instr_id in instructions {
            if let Instruction::Call { func, .. } = &main_fn.dfg[*instr_id]
                && let Value::Function(callee_id) = &main_fn.dfg[*func]
            {
                // Should be calling a new specialized function, not the original
                let callee = &ssa.functions[callee_id];
                // The specialized function should have fewer blocks (branch folded)
                let block_count = callee.reachable_blocks().len();
                assert!(
                    block_count < 4,
                    "Expected specialized function to have fewer blocks, got {block_count}"
                );
                found_specialized_call = true;
            }
        }
        assert!(found_specialized_call, "Expected a call to a specialized function");
    }

    #[test]
    fn mixed_constant_and_variable_args() {
        // Only the constant positions should be replaced in the clone.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 5, v0, u32 10)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field, v2: u32):
            v3 = eq v0, u32 5
            jmpif v3 then: b1(), else: b2()
          b1():
            v4 = add v1, Field 1
            jmp b3(v4)
          b2():
            v5 = sub v1, Field 1
            jmp b3(v5)
          b3(v6: Field):
            constrain v6 == Field 0
            return
        }
        ";

        let ssa = run_specialization(src);

        // Should have a specialized clone (original + specialized = at least 3 functions including main)
        assert!(
            ssa.functions.len() >= 3,
            "Expected at least 3 functions (main + original + specialized)"
        );
    }

    #[test]
    fn no_constants_no_specialization() {
        // If no arguments are constant, no specialization should happen.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: Field):
            call f1(v0, v1)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, Field 1
            jmp b3(v3)
          b2():
            v4 = sub v1, Field 1
            jmp b3(v4)
          b3(v5: Field):
            constrain v5 == Field 0
            return
        }
        ";

        let ssa = run_specialization(src);

        // Should have exactly 2 functions (main + original, no clone).
        assert_eq!(ssa.functions.len(), 2, "Expected no specialization when no constants");
    }

    #[test]
    fn multiple_constant_patterns_create_separate_clones() {
        // Same callee called with different constant patterns should create separate clones.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 1, v0)
            call f1(u32 2, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, Field 1
            jmp b3(v3)
          b2():
            v4 = sub v1, Field 1
            jmp b3(v4)
          b3(v5: Field):
            constrain v5 == Field 0
            return
        }
        ";

        let ssa = run_specialization(src);

        // Should have at least 4 functions: main + original + 2 specialized clones.
        assert!(
            ssa.functions.len() >= 4,
            "Expected at least 4 functions for two different constant patterns, got {}",
            ssa.functions.len()
        );
    }

    #[test]
    fn recursive_function_not_specialized() {
        // Recursive Brillig functions should not be specialized.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = call f1(u32 5) -> Field
            return
        }
        brillig(inline) fn recursive_fn f1 {
          b0(v0: u32):
            v1 = eq v0, u32 0
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(Field 0)
          b2():
            v3 = sub v0, u32 1
            v4 = call f1(v3) -> Field
            v5 = add v4, Field 1
            jmp b3(v5)
          b3(v2: Field):
            return v2
        }
        ";

        let ssa = run_specialization(src);

        // Should have exactly 2 functions (main + original, no clone).
        assert_eq!(ssa.functions.len(), 2, "Expected no specialization for recursive function");
    }

    #[test]
    fn disabled_via_zero_threshold() {
        // Setting threshold to 0 should disable the pass entirely.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 1, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, Field 1
            jmp b3(v3)
          b2():
            v4 = sub v1, Field 1
            jmp b3(v4)
          b3(v5: Field):
            constrain v5 == Field 0
            return
        }
        ";

        let ssa = run_specialization_with_options(src, 0, 3);

        // Should have exactly 2 functions (main + original, no clone).
        assert_eq!(ssa.functions.len(), 2, "Expected pass to be disabled with threshold=0");
    }

    #[test]
    fn clone_cap_limits_specializations() {
        // With max_specializations_per_fn=1, only one clone should be created even with
        // multiple constant patterns.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 1, v0)
            call f1(u32 2, v0)
            call f1(u32 3, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, Field 1
            jmp b3(v3)
          b2():
            v4 = sub v1, Field 1
            jmp b3(v4)
          b3(v5: Field):
            constrain v5 == Field 0
            return
        }
        ";

        let ssa = run_specialization_with_options(src, 1, 1);

        // Should have at most 3 functions: main + original + 1 specialized clone.
        // (threshold=1 means even small improvements count, but only 1 clone allowed)
        assert!(
            ssa.functions.len() <= 3,
            "Expected at most 3 functions with max_per_fn=1, got {}",
            ssa.functions.len()
        );
    }

    #[test]
    fn below_threshold_no_specialization() {
        // If the function body doesn't benefit enough from the constant,
        // the clone should be discarded.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(u32 1, v0)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            v2 = add v1, Field 1
            v3 = add v2, Field 2
            v4 = add v3, Field 3
            v5 = add v4, Field 4
            v6 = add v5, Field 5
            v7 = add v6, Field 6
            v8 = add v7, Field 7
            v9 = add v8, Field 8
            constrain v9 == Field 0
            return
        }
        ";

        // With a high threshold (99%), the tiny benefit of knowing v0 is constant
        // in a function that doesn't use v0 in any branch should not be enough.
        let ssa = run_specialization_with_options(src, 99, 3);

        assert_eq!(ssa.functions.len(), 2, "Expected no specialization below threshold");
    }

    #[test]
    fn nested_calls_rewritten_in_clones() {
        // When `foo` calls `bar` with constants, clones of `foo` should call the
        // specialized `bar` clone, not the original `bar`.
        //
        // main calls foo(1, v0) and foo(v0, 2)
        // foo calls bar(3, v1) and bar(v1, 4)
        // -> clones of foo should call the specialized bar, not original bar
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            call f1(u32 1, v0)
            call f1(v0, u32 2)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: u32):
            v2 = eq v0, u32 1
            jmpif v2 then: b1(), else: b2()
          b1():
            call f2(u32 3, v1)
            jmp b3()
          b2():
            call f2(v1, u32 4)
            jmp b3()
          b3():
            return
        }
        brillig(inline) fn bar f2 {
          b0(v0: u32, v1: u32):
            v2 = eq v0, u32 3
            jmpif v2 then: b1(), else: b2()
          b1():
            v3 = add v1, u32 10
            jmp b3(v3)
          b2():
            v4 = sub v1, u32 10
            jmp b3(v4)
          b3(v5: u32):
            constrain v5 == u32 0
            return
        }
        ";

        let ssa = run_specialization_with_options(src, 1, 5);
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32):
            call f3(u32 1, v0)
            call f1(v0, u32 2)
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: u32):
            v3 = eq v0, u32 1
            jmpif v3 then: b1(), else: b2()
          b1():
            call f4(u32 3, v1)
            jmp b3()
          b2():
            call f5(v1, u32 4)
            jmp b3()
          b3():
            return
        }
        brillig(inline) fn bar f2 {
          b0(v0: u32, v1: u32):
            v4 = eq v0, u32 3
            jmpif v4 then: b1(), else: b2()
          b1():
            v7 = add v1, u32 10
            jmp b3(v7)
          b2():
            v6 = sub v1, u32 10
            jmp b3(v6)
          b3(v2: u32):
            constrain v2 == u32 0
            return
        }
        brillig(inline) fn foo f3 {
          b0(v0: u32, v1: u32):
            call f4(u32 3, v1)
            return
        }
        brillig(inline) fn bar f4 {
          b0(v0: u32, v1: u32):
            v3 = add v1, u32 10
            constrain v3 == u32 0
            return
        }
        brillig(inline) fn bar f5 {
          b0(v0: u32, v1: u32):
            v4 = eq v0, u32 3
            jmpif v4 then: b1(), else: b2()
          b1():
            jmp b3(u32 14)
          b2():
            v7 = sub u32 4, u32 10
            jmp b3(v7)
          b3(v2: u32):
            constrain v2 == u32 0
            return
        }
        ");
    }
}
