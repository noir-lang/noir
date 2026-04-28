//! Analyzes the purity of each function and tag each function call with that function's purity.
//! This is purely an analysis pass on its own but can help future optimizations.
//!
//! There is no constraint on when this pass needs to be run, but it is generally more
//! beneficial to perform this pass before inlining or loop unrolling so that it can:
//! 1. Run faster by processing fewer instructions.
//! 2. Be run earlier in the pass list so that more passes afterward can use the results of
//!    this pass.
//!
//! Performing this pass after defunctionalization may also help more function calls be
//! identified as calling known pure functions.

use std::sync::Arc;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ir::call_graph::CallGraph;
use crate::ssa::ir::types::Type;
use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::{Instruction, TerminatorInstruction},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Analyzes the purity of each function and tag each function call with that function's purity.
    /// This is purely an analysis pass on its own but can help future optimizations.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn purity_analysis(mut self) -> Ssa {
        let call_graph = CallGraph::from_ssa(&self);

        let (sccs, recursive_functions) = call_graph.sccs();

        // First look through each function to get a baseline on its purity and collect
        // the functions it calls to build a call graph.
        let purities: HashMap<_, _> =
            self.functions.values().map(|function| (function.id(), function.is_pure())).collect();

        // Then transitively 'infect' any functions which call impure functions as also
        // impure.
        let purities = analyze_call_graph(call_graph, purities, &sccs, &recursive_functions);
        let purities = Arc::new(purities);

        // We're done, now store purities somewhere every dfg can find it.
        for function in self.functions.values_mut() {
            function.dfg.set_function_purities(purities.clone());
        }

        #[cfg(debug_assertions)]
        purity_analysis_post_check(&self);

        self
    }
}

/// Post-check condition for [Ssa::purity_analysis].
///
/// Succeeds if:
///   - all functions have a purity status attached to it.
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn purity_analysis_post_check(ssa: &Ssa) {
    if let Some((id, _)) =
        ssa.functions.iter().find(|(id, function)| function.dfg.purity_of(**id).is_none())
    {
        panic!("Function {id} does not have a purity status")
    }
}

pub(crate) type FunctionPurities = HashMap<FunctionId, Purity>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Purity {
    /// Function is completely pure and doesn't rely on a predicate at all.
    /// Pure functions can be freely deduplicated or even removed from the program.
    Pure,

    /// Function is mostly pure. As long as the predicate is the same.
    /// This applies to functions with `constrain` in them. So long as their
    /// parameters are the same, the `constrain` should be to the same values
    /// so the function is conceptually pure from a deduplication perspective
    /// even though it can still interact with the `enable_side_effects`/predicate variable.
    ///
    /// PureWithPredicate functions can only be deduplicated with identical predicates
    /// or a predicate that is a subset of the original.
    PureWithPredicate,

    /// This function is impure and cannot be deduplicated even with identical inputs.
    /// This is most commonly the case for any function taking or returning a
    /// reference value.
    Impure,
}

impl Purity {
    /// Unifies two purity values, returning the lower common denominator of the two
    pub(crate) fn unify(self, other: Purity) -> Purity {
        match (self, other) {
            (Purity::Pure, Purity::Pure) => Purity::Pure,
            (Purity::Impure, _) | (_, Purity::Impure) => Purity::Impure,
            _ => Purity::PureWithPredicate,
        }
    }
}

impl std::fmt::Display for Purity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Purity::Pure => write!(f, "pure"),
            Purity::PureWithPredicate => write!(f, "predicate_pure"),
            Purity::Impure => write!(f, "impure"),
        }
    }
}

impl Function {
    fn is_pure(&self) -> Purity {
        let contains_reference = |value_id: &ValueId| {
            let typ = self.dfg.type_of_value(*value_id);
            typ.contains_reference()
        };

        if self.parameters().iter().any(&contains_reference) {
            return Purity::Impure;
        }

        // Collect all parameters that are arrays or contain arrays, but only for Brillig.
        // If we detect an array_set potentially operating on a brillig array input, the entire
        // function becomes impure.
        let brillig_array_inputs = if self.runtime().is_brillig() {
            self.parameters()
                .iter()
                .filter(|param| {
                    let typ = self.dfg.type_of_value(**param);
                    typ.contains_an_array()
                })
                .collect::<HashSet<_>>()
        } else {
            HashSet::default()
        };
        let has_brillig_array_input = !brillig_array_inputs.is_empty();

        // Records whether there's an `array_set`, `inc_rc` or `dec_rc` in this function.
        let mut has_array_set_or_rc = false;

        // Records whether a brillig array input was used in an instruction that could have moved
        // it to another value. Examples include `store`, `array_set`, and even `array_get` for parameters
        // that have nested arrays.
        let mut brillig_array_input_was_moved = false;

        let mut result = if self.runtime().is_acir() {
            Purity::Pure
        } else {
            // Because we return bogus values when a brillig function is called from acir
            // in a disabled predicate, brillig functions can never be truly pure unfortunately.
            Purity::PureWithPredicate
        };

        for block in self.reachable_blocks() {
            for instruction in self.dfg[block].instructions() {
                // We don't defer to Instruction::can_be_deduplicated, Instruction::requires_acir_gen_predicate,
                // etc. since we don't consider local mutations to be impure. Local mutations should
                // be invisible to calling functions so as long as no references are taken as
                // parameters or returned, we can ignore them.
                // We even ignore Constrain instructions. As long as the external parameters are
                // identical, we should be constraining the same values anyway.
                let ins = &self.dfg[*instruction];
                match ins {
                    Instruction::Constrain(..)
                    | Instruction::ConstrainNotEqual(..)
                    | Instruction::RangeCheck { .. } => result = Purity::PureWithPredicate,

                    // These instructions may be pure unless:
                    // - We may divide by zero
                    // - The array index is out of bounds.
                    // For both cases we can still treat them as pure if the arguments are known
                    // constants.
                    Instruction::Binary(_) | Instruction::ArrayGet { .. } => {
                        if ins.requires_acir_gen_predicate(&self.dfg) {
                            result = Purity::PureWithPredicate;
                        }
                    }
                    Instruction::ArraySet { .. } => {
                      has_array_set_or_rc = true;
                      result = Purity::PureWithPredicate;
                    }
                    Instruction::Call { func, .. } => {
                        match &self.dfg[*func] {
                            Value::Function(_) => {
                                // We don't know if this function is pure or not yet,
                                //
                                // `is_pure` is intended to be called on each function, building
                                // up a call graph of sorts to check afterwards to propagate impurity
                                // from called functions to their callers. Therefore, an initial "Pure"
                                // result here could be overridden by one of these dependencies being impure.
                            }
                            Value::Intrinsic(intrinsic) => match intrinsic.purity() {
                                Purity::Pure => (),
                                Purity::PureWithPredicate => result = Purity::PureWithPredicate,
                                Purity::Impure => return Purity::Impure,
                            },
                            Value::ForeignFunction(_) => return Purity::Impure,
                            // The function we're calling is unknown in the remaining cases,
                            // so just assume the worst.
                            Value::Global(_)
                            | Value::Instruction { .. }
                            | Value::Param { .. }
                            | Value::NumericConstant { .. } => return Purity::Impure,
                        }
                    }

                    // The rest are always pure (including allocate, load, & store)
                    Instruction::Cast(_, _)
                    | Instruction::Not(_)
                    | Instruction::Truncate { .. }
                    | Instruction::Allocate
                    // Load and store are considered pure since there is a separate check ensuring
                    // no parameters or return values are references. With this check, we can be
                    // sure any load/store is purely local.
                    | Instruction::Load { .. }
                    | Instruction::Store { .. }
                    | Instruction::EnableSideEffectsIf { .. }
                    | Instruction::IfElse { .. }
                    | Instruction::MakeArray { .. }
                    | Instruction::Noop => (),

                    Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => {
                        has_array_set_or_rc = true;
                    }
                }

                // Separately, check if any instruction could be moving a Brillig array input.
                if has_brillig_array_input {
                    match ins {
                        Instruction::Binary(_)
                        | Instruction::Cast(..)
                        | Instruction::Not(_)
                        | Instruction::Truncate { .. }
                        | Instruction::Constrain(..)
                        | Instruction::ConstrainNotEqual(..)
                        | Instruction::RangeCheck { .. }
                        | Instruction::Call { .. }
                        | Instruction::Allocate
                        | Instruction::EnableSideEffectsIf { .. }
                        | Instruction::Noop => {
                            // This can't possibly move a Brillig array input.
                            // A `call` could mutate a Brillig array input, but if that is the case
                            // the the call itself will be marked as impure, and so then this function will
                            // be impure... but that is a check that is done later on.
                        }

                        Instruction::Load { .. }
                        | Instruction::Store { .. }
                        | Instruction::ArraySet { .. }
                        | Instruction::IncrementRc { .. }
                        | Instruction::DecrementRc { .. }
                        | Instruction::IfElse { .. }
                        | Instruction::MakeArray { .. } => {
                            // Check if any of these instructions is operating on a Brillig array input
                            brillig_array_input_was_moved |= has_brillig_array_input
                                && ins.any_value(|value| brillig_array_inputs.contains(&value));
                        }
                        Instruction::ArrayGet { array, index: _ } => {
                            // For ArrayGet we do something slightly different: if it operates on a Brillig array input
                            // array, an array could be moved if it's nested inside `array` (for example if the type
                            // is `[[Field; 2]; 3]`. However, if the `array` is an array without nested arrays, no
                            // array will be moved here. We consider this case specifically because fetching from a
                            // non-nested Brillig array input is a common pattern.
                            if brillig_array_inputs.contains(array) {
                                let typ = self.dfg.type_of_value(*array);
                                let typ = typ.as_ref();
                                match typ {
                                    Type::Array(items, _) | Type::Vector(items) => {
                                        if items.iter().any(|item| item.contains_an_array()) {
                                            brillig_array_input_was_moved = true;
                                        }
                                    }
                                    Type::Numeric(_) | Type::Reference(_, _) | Type::Function => (),
                                }
                            }
                        }
                    }
                }
            }

            // If the function returns a reference it is impure
            let terminator = self.dfg[block].terminator();
            if let Some(terminator) = terminator {
                if let TerminatorInstruction::Return { return_values, .. } = terminator
                    && return_values.iter().any(&contains_reference)
                {
                    return Purity::Impure;
                }

                // Also check if any Brillig array input is moved in a terminator
                if has_brillig_array_input
                    && terminator.any_value(|value| brillig_array_inputs.contains(&value))
                {
                    brillig_array_input_was_moved = true;
                }
            }
        }

        // If a Brillig array input was moved, and we found any instruction that could mutate it
        // (`array_set`, `inc_rc` or `dec_rc`) then we consider the function impure.
        if has_array_set_or_rc && brillig_array_input_was_moved {
            return Purity::Impure;
        }

        result
    }
}

fn analyze_call_graph(
    call_graph: CallGraph,
    starting_purities: FunctionPurities,
    sccs: &[Vec<FunctionId>],
    recursive_functions: &HashSet<FunctionId>,
) -> FunctionPurities {
    let mut finished = HashMap::default();

    // Map FunctionId -> SCC index for quick lookup
    let mut func_to_scc = HashMap::default();
    for (i, scc) in sccs.iter().enumerate() {
        for &func in scc {
            // Each function belongs to exactly one SCC by definition of SCCs.
            // Therefore inserting into func_to_scc here is safe, and there will
            // be no overwrites.
            let inserted = func_to_scc.insert(func, i);
            assert!(inserted.is_none(), "Function appears in multiple SCCs");
        }
    }

    // Track SCC purity
    let mut scc_purities: Vec<Purity> = sccs
        .iter()
        .map(|scc| scc.iter().map(|f| starting_purities[f]).fold(Purity::Pure, |a, b| a.unify(b)))
        .collect();

    // Iteratively propagate purity between SCCs until convergence
    let mut changed = true;
    while changed {
        changed = false;

        for (i, scc) in sccs.iter().enumerate() {
            let mut combined_purity = scc_purities[i];

            // Look at neighbors outside the SCC
            for &func in scc {
                let idx = call_graph.ids_to_indices()[&func];
                for neighbor_idx in call_graph.graph().neighbors(idx) {
                    let neighbor = call_graph.indices_to_ids()[&neighbor_idx];
                    let neighbor_scc = func_to_scc[&neighbor];
                    if neighbor_scc != i {
                        combined_purity = combined_purity.unify(scc_purities[neighbor_scc]);
                    }
                }

                // Recursive functions cannot be fully pure (may recurse indefinitely),
                // but we still treat them as PureWithPredicate for deduplication purposes.
                // If we were to mark recursive functions pure we may entirely eliminate an infinite loop.
                if recursive_functions.contains(&func) {
                    combined_purity = combined_purity.unify(Purity::PureWithPredicate);
                }
            }

            if combined_purity != scc_purities[i] {
                scc_purities[i] = combined_purity;
                changed = true;
            }
        }
    }

    // Assign SCC purity to all functions in the SCC
    for (i, scc) in sccs.iter().enumerate() {
        for &func in scc {
            finished.insert(func, scc_purities[i]);
        }
    }

    finished
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{ir::function::FunctionId, opt::pure::Purity, ssa_gen::Ssa},
    };

    use test_case::test_case;

    #[test]
    fn classify_functions() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = allocate -> &mut Field
                call f1(v0)
                v1 = call f2() -> &mut Field
                call f3(Field 0)
                call f4()
                call f5()
                call f6()
                v2 = call f7(u32 2) -> u32
                return
            }

            acir(inline) fn impure_take_ref f1 {
              b0(v0: &mut Field):
                return
            }

            acir(inline) fn impure_returns_ref f2 {
              b0():
                v0 = allocate -> &mut Field
                return v0
            }

            acir(inline) fn predicate_constrain f3 {
              b0(v0: Field):
                constrain v0 == Field 0
                return
            }

            acir(inline) fn predicate_calls_predicate f4 {
              b0():
                call f3(Field 0)
                return
            }

            acir(inline) fn predicate_oob f5 {
              b0():
                v0 = make_array [Field 0, Field 1] : [Field; 2]
                v1 = array_get v0, index u32 2 -> Field
                return
            }

            acir(inline) fn pure_basic f6 {
              b0():
                v0 = make_array [Field 0, Field 1] : [Field; 2]
                v1 = array_get v0, index u32 1 -> Field
                v2 = allocate -> &mut Field
                store Field 0 at v2
                return
            }

            acir(inline) fn pure_recursive f7 {
              b0(v0: u32):
                v1 = lt v0, u32 1
                jmpif v1 then: b1(), else: b2()
              b1():
                jmp b3(u32 0)
              b2():
                v3 = call f7(v0) -> u32
                call f6()
                jmp b3(v3)
              b3(v4: u32):
                return v4
            }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(3)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(4)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(5)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(6)], Purity::Pure);
        assert_eq!(purities[&FunctionId::test_new(7)], Purity::PureWithPredicate);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            call f1(v0)
            v3 = call f2() -> &mut Field
            call f3(Field 0)
            call f4()
            call f5()
            call f6()
            v11 = call f7(u32 2) -> u32
            return
        }
        acir(inline) impure fn impure_take_ref f1 {
          b0(v0: &mut Field):
            return
        }
        acir(inline) impure fn impure_returns_ref f2 {
          b0():
            v0 = allocate -> &mut Field
            return v0
        }
        acir(inline) predicate_pure fn predicate_constrain f3 {
          b0(v0: Field):
            constrain v0 == Field 0
            return
        }
        acir(inline) predicate_pure fn predicate_calls_predicate f4 {
          b0():
            call f3(Field 0)
            return
        }
        acir(inline) predicate_pure fn predicate_oob f5 {
          b0():
            v2 = make_array [Field 0, Field 1] : [Field; 2]
            v4 = array_get v2, index u32 2 -> Field
            return
        }
        acir(inline) pure fn pure_basic f6 {
          b0():
            v2 = make_array [Field 0, Field 1] : [Field; 2]
            v4 = array_get v2, index u32 1 -> Field
            v5 = allocate -> &mut Field
            store Field 0 at v5
            return
        }
        acir(inline) predicate_pure fn pure_recursive f7 {
          b0(v0: u32):
            v3 = lt v0, u32 1
            jmpif v3 then: b1(), else: b2()
          b1():
            jmp b3(u32 0)
          b2():
            v5 = call f7(v0) -> u32
            call f6()
            jmp b3(v5)
          b3(v1: u32):
            return v1
        }
        ");
    }

    #[test]
    fn regression_8625() {
        // This test checks for a case which would result in some functions not having a purity status applied.
        // See https://github.com/noir-lang/noir/issues/8625
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: [u8; 3]):
            inc_rc v0
            v1 = allocate -> &mut [u8; 3]
            store v0 at v1
            inc_rc v0
            inc_rc v0
            call f1(v1, u32 0, u32 2, Field 3)
            return
        }
        brillig(inline) fn impure_because_reference_arg f1 {
          b0(v0: &mut [u8; 3], v1: u32, v2: u32, v3: Field):
            call f2(v0, v1, v2, v3)
            return
        }
        brillig(inline) fn also_impure_because_reference_arg f2 {
          b0(v0: &mut [u8; 3], v1: u32, v2: u32, v3: Field):
            call f3()
            return
        }
        brillig(inline) fn pure f3 {
          b0():
            return
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(3)], Purity::PureWithPredicate);
    }

    #[test]
    fn handles_unreachable_functions() {
        // Regression test for https://github.com/noir-lang/noir/issues/8666
        let src = r#"
        brillig(inline) fn main f0 {
          b0():
            return
        }
        brillig(inline) fn func_1 f1 {
          b0():
            return
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }

    /// Functions using inc_rc or dec_rc are always impure - see constant_folding::do_not_deduplicate_call_with_inc_rc
    /// as an example of a case in which semantics are changed if these are considered pure.
    #[test]
    fn inc_rc_is_impure() {
        // This test ensures that a function which mutates an array pointer is marked impure.
        // This protects against future deduplication passes incorrectly assuming purity.
        let src = r#"
        brillig(inline) fn mutator f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        brillig(inline) fn mutator f1 {
          b0(v0: [Field; 2]):
            dec_rc v0  // We wouldn't produce this code. This is just to ensure dec_rc is impure.
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        "#;

        let ssa = Ssa::from_str_no_validation(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
    }

    #[test]
    fn brillig_array_set_is_impure() {
        let src = r#"
        brillig(inline) fn mutator f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        // We wouldn't produce this code. This is to ensure `array_set` on a function parameter is marked impure.
        brillig(inline) fn mutator f1 {
          b0(v0: [Field; 2]):
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
    }

    #[test]
    fn brillig_array_set_on_local_array_pure() {
        let src = r#"
        brillig(inline) fn mutator f0 {
          b0(v0: [Field; 2]):
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        brillig(inline) fn mutator f1 {
          b0():
            v2 = make_array [Field 1, Field 2] : [Field; 2]
            v5 = array_set v2, index u32 0, value Field 5
            return v5
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        // Brillig functions have a starting purity of PureWithPredicate
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }

    #[test]
    fn direct_brillig_recursion_marks_functions_pure_with_predicate() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        brillig(inline) fn f1 f1 {
          b0():
            call f1()
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }

    #[test]
    fn mutual_recursion_marks_functions_pure() {
        // We want to test that two pure mutually recursive functions do in fact mark each other as PureWithPredicate.
        // If we have indefinite recursion and we may accidentally eliminate an infinite loop before inlining can catch it.
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = call f1(u32 4) -> bool
            return
        }
        acir(inline) fn is_even f1 {
          b0(v0: u32):
            v1 = eq v0, u32 0
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(u1 1)
          b2():
            v2 = unchecked_sub v0, u32 1
            v3 = call f2(v2) -> bool
            jmp b3(v3)
          b3(v4: bool):
            return v4
        }
        acir(inline) fn is_odd f2 {
          b0(v0: u32):
            v1 = eq v0, u32 0
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(u1 0)
          b2():
            v2 = unchecked_sub v0, u32 1
            v3 = call f1(v2) -> bool
            jmp b3(v3)
          b3(v4: bool):
            return v4
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::PureWithPredicate);
    }

    /// This test matches [mutual_recursion_marks_functions_pure] except all functions have a Brillig runtime
    #[test]
    fn brillig_mutual_recursion_marks_functions_pure_with_predicate() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v0 = call f1(u32 4) -> bool
            return
        }
        brillig(inline) fn is_even f1 {
          b0(v0: u32):
            v1 = eq v0, u32 0
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(u1 1)
          b2():
            v2 = unchecked_sub v0, u32 1
            v3 = call f2(v2) -> bool
            jmp b3(v3)
          b3(v4: bool):
            return v4
        }
        brillig(inline) fn is_odd f2 {
          b0(v0: u32):
            v1 = eq v0, u32 0
            jmpif v1 then: b1(), else: b2()
          b1():
            jmp b3(u1 0)
          b2():
            v2 = unchecked_sub v0, u32 1
            v3 = call f1(v2) -> bool
            jmp b3(v3)
          b3(v4: bool):
            return v4
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::PureWithPredicate);
    }

    #[test]
    fn mutual_recursion_marks_functions_impure() {
        // f1 -> f2 -> f3 -> f1 (a cycle of three functions)
        // Only f3 is locally impure (it returns a reference).
        // All three must be marked Impure.
        //
        // We call f2 in main as we want the DFS to not look at f3 first (which is "Impure").
        // If f3 is found first the cycle will get correctly marked as impure.
        // We want to make sure that even when the first function in the recursive cycle
        // is not marked as impure that we still accurately mark the entire cycle impure.
        // Calling f2 first, means the cycle will look at f1 first, which still
        // has a starting purity of "Pure".
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = call f2() -> Field
            return
        }
        acir(inline) fn f1 f1 {
          b0():
            v0 = call f2() -> Field
            return v0
        }
        acir(inline) fn f2 f2 {
          b0():
            v0 = call f3() -> &mut Field
            v1 = load v0 -> Field
            return v1
        }
        acir(inline) fn f3 f3 {
          b0():
            v0 = call f1() -> Field
            v1 = allocate -> &mut Field
            return v1
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        // All must be impure due to the cycle involved f3 when returns a reference.
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(3)], Purity::Impure);
    }

    /// This test matches [mutual_recursion_marks_functions_impure] except all functions have a Brillig runtime
    #[test]
    fn brillig_mutual_recursion_marks_functions_impure() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0():
            v0 = call f2() -> Field
            return
        }
        brillig(inline) fn f1 f1 {
          b0():
            v0 = call f2() -> Field
            return v0
        }
        brillig(inline) fn f2 f2 {
          b0():
            v0 = call f3() -> &mut Field
            v1 = load v0 -> Field
            return v1
        }
        brillig(inline) fn f3 f3 {
          b0():
            v0 = call f1() -> Field
            v1 = allocate -> &mut Field
            return v1
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        // All must be impure due to the cycle involved f3 when returns a reference.
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(3)], Purity::Impure);
    }

    #[test]
    fn brillig_functions_are_pure_with_predicate_if_they_are_an_entry_point() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            call f1()
            call f1()
            return
        }
        brillig(inline) fn pure_basic f1 {
          b0():
            v2 = make_array [Field 0, Field 1] : [Field; 2]
            v4 = array_get v2, index u32 1 -> Field
            v5 = allocate -> &mut Field
            store Field 0 at v5
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }

    #[test]
    fn brillig_functions_are_pure_with_predicate_if_they_are_not_an_entry_point() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            call f1()
            call f1()
            return
        }
        brillig(inline) fn pure_basic f1 {
          b0():
            v2 = make_array [Field 0, Field 1] : [Field; 2]
            v4 = array_get v2, index u32 1 -> Field
            v5 = allocate -> &mut Field
            store Field 0 at v5
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);

        // Note: even though it would be fine to mark f1 as pure, something in Aztec-Packages
        // gets broken so until we figure out what that is we can't mark these as pure.
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }

    #[test]
    fn call_to_function_value() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v5 = make_array [f1, f2] : [function; 2]
            v7 = lt v0, u32 2
            constrain v7 == u1 1, "Index out of bounds"
            v9 = array_get v5, index v0 -> function
            call v9()
            return
        }
        acir(inline) fn lambda f1 {
          b0():
            return
        }
        acir(inline) fn lambda f2 {
          b0():
            return
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        // Even though the functions referenced by the function values are pure
        // we assume the worse case for functions containing calls to function values.
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Pure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Pure);
    }

    #[test_case("ecdsa_secp256k1")]
    #[test_case("ecdsa_secp256r1")]
    fn marks_ecdsa_verification_as_pure_with_predicate(ecdsa_func: &str) {
        let src = format!(
            r#"
        acir(inline) fn main f0 {{
            b0(v0: [u8; 32], v1: [u8; 32], v2: [u8; 64], v3: [u8; 32]):
            v4 = call {ecdsa_func}(v0, v1, v2, v3, u1 1) -> u1
            return
        }}
        "#
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.main().dfg.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
    }

    #[test]
    fn considers_array_set_to_any_array_as_impure_if_entry_point_has_an_array() {
        // `v6 = array_set v2, ...` ends up operating on `v0` because it's being passed
        // in `b1(...)` as `v2`. So even if `array_set v2` doesn't directly operate on a function parameter,
        // it can end up operating on one, indirectly. This test ensures we catch this case.
        let src = "
        brillig(inline_never) fn f f1 {
          b0(v0: [u1; 1]):
            jmp b1(u32 0, v0)
          b1(v1: u32, v2: [u1; 1]):
            v4 = eq v1, u32 0
            jmpif v4 then: b2(), else: b3()
          b2():
            v6 = array_set v2, index u32 0, value u1 0
            v8 = unchecked_add v1, u32 1
            jmp b1(v8, v6)
          b3():
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline_never) impure fn f f0 {
          b0(v0: [u1; 1]):
            jmp b1(u32 0, v0)
          b1(v1: u32, v2: [u1; 1]):
            v4 = eq v1, u32 0
            jmpif v4 then: b2(), else: b3()
          b2():
            v6 = array_set v2, index u32 0, value u1 0
            v8 = unchecked_add v1, u32 1
            jmp b1(v8, v6)
          b3():
            return v2
        }
        ");
    }

    #[test]
    fn does_not_consider_impure_if_brillig_array_input_is_not_moved_even_though_there_is_an_array_set()
     {
        // Even though there's an array_set, which *could* operate on a brillig array input,
        // we notice that v0 is never moved around so it can't be the target of any array_set.
        // v0 is used in an `array_get`, but since v0 is an array and doesn't have nested arrays
        // in it, no array is actually moved.
        let src = "
        brillig(inline_never) fn f f0 {
          b0(v0: [u1; 1]):
            v4 = make_array [u1 0] : [u1; 1]
            v6 = array_get v0, index u32 0 -> u1
            jmp b1(u32 0, v4)
          b1(v1: u32, v2: [u1; 1]):
            v7 = eq v1, u32 0
            jmpif v7 then: b2(), else: b3()
          b2():
            v8 = array_set v2, index u32 0, value u1 0
            v10 = unchecked_add v1, u32 1
            jmp b1(v10, v8)
          b3():
            return v2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline_never) predicate_pure fn f f0 {
          b0(v0: [u1; 1]):
            v4 = make_array [u1 0] : [u1; 1]
            v6 = array_get v0, index u32 0 -> u1
            jmp b1(u32 0, v4)
          b1(v1: u32, v2: [u1; 1]):
            v7 = eq v1, u32 0
            jmpif v7 then: b2(), else: b3()
          b2():
            v8 = array_set v2, index u32 0, value u1 0
            v10 = unchecked_add v1, u32 1
            jmp b1(v10, v8)
          b3():
            return v2
        }
        ");
    }

    #[test]
    fn considers_impure_if_brillig_input_array_is_stored_and_there_is_an_array_set() {
        let src = "
        brillig(inline_never) fn f f0 {
          b0(v0: [u1; 1]):
            v1 = allocate -> &mut [u1; 1]
            store v0 at v1
            v2 = load v1 -> [u1; 1]
            v5 = array_set v2, index u32 0, value u1 0
            return v0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline_never) impure fn f f0 {
          b0(v0: [u1; 1]):
            v1 = allocate -> &mut [u1; 1]
            store v0 at v1
            v2 = load v1 -> [u1; 1]
            v5 = array_set v2, index u32 0, value u1 0
            return v0
        }
        ");
    }

    #[test]
    fn considers_impure_if_brillig_input_array_is_stored_and_there_is_an_inc_rc() {
        let src = "
        brillig(inline_never) fn f f0 {
          b0(v0: [u1; 1]):
            v1 = allocate -> &mut [u1; 1]
            store v0 at v1
            v2 = load v1 -> [u1; 1]
            inc_rc v2
            return v0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline_never) impure fn f f0 {
          b0(v0: [u1; 1]):
            v1 = allocate -> &mut [u1; 1]
            store v0 at v1
            v2 = load v1 -> [u1; 1]
            inc_rc v2
            return v0
        }
        ");
    }

    #[test]
    fn considers_impure_if_brillig_input_nested_array_is_moved_and_there_is_a_dec_rc() {
        let src = "
        brillig(inline_never) fn f f0 {
          b0(v0: [[u1; 1]; 1]):
            v1 = array_get v0, index u32 0 -> [u1; 1]
            inc_rc v1
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline_never) impure fn f f0 {
          b0(v0: [[u1; 1]; 1]):
            v2 = array_get v0, index u32 0 -> [u1; 1]
            inc_rc v2
            return
        }
        ");
    }
}
