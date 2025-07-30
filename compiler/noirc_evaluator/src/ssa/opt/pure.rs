use fxhash::FxHashMap as HashMap;
use petgraph::visit::DfsPostOrder;

use crate::ssa::ir::call_graph::CallGraph;
use crate::ssa::{
    ir::{
        function::{Function, FunctionId},
        instruction::{Instruction, TerminatorInstruction},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Analyze the purity of each function and tag each function call with that function's purity.
    /// This is purely an analysis pass on its own but can help future optimizations.
    ///
    /// There is no constraint on when this pass needs to be run, but it is generally more
    /// beneficial to perform this pass before inlining or loop unrolling so that it can:
    /// 1. Run faster by processing fewer instructions.
    /// 2. Be run earlier in the pass list so that more passes afterward can use the results of
    ///    this pass.
    ///
    /// Performing this pass after defunctionalization may also help more function calls be
    /// identified as calling known pure functions.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn purity_analysis(mut self) -> Ssa {
        // First look through each function to get a baseline on its purity and collect
        // the functions it calls to build a call graph.
        let functions = self.functions.values();
        let purities: FunctionPurities = functions
            .map(|function| (function.id(), function.is_pure(&self.function_purities)))
            .collect();

        // Then transitively 'infect' any functions which call impure functions as also
        // impure.
        let call_graph = CallGraph::from_ssa(&self);
        let purities = analyze_call_graph(call_graph, purities);

        // We're done, now store purities
        self.function_purities = purities;

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
    if let Some(id) = ssa.functions.keys().find(|id| !ssa.function_purities.contains_key(*id)) {
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
    fn is_pure(&self, purities: &FunctionPurities) -> Purity {
        let contains_reference = |value_id: &ValueId| {
            let typ = self.dfg.type_of_value(*value_id);
            typ.contains_reference()
        };

        if self.parameters().iter().any(&contains_reference) {
            return Purity::Impure;
        }

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
                match &self.dfg[*instruction] {
                    Instruction::Constrain(..)
                    | Instruction::ConstrainNotEqual(..)
                    | Instruction::RangeCheck { .. } => result = Purity::PureWithPredicate,

                    // These instructions may be pure unless:
                    // - We may divide by zero
                    // - The array index is out of bounds.
                    // For both cases we can still treat them as pure if the arguments are known
                    // constants.
                    ins @ (Instruction::Binary(_)
                    | Instruction::ArrayGet { .. }
                    | Instruction::ArraySet { .. }) => {
                        if ins.requires_acir_gen_predicate(&self.dfg, purities) {
                            result = Purity::PureWithPredicate;
                        }
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
                    | Instruction::Load { .. }
                    | Instruction::Store { .. }
                    | Instruction::EnableSideEffectsIf { .. }
                    | Instruction::IncrementRc { .. }
                    | Instruction::DecrementRc { .. }
                    | Instruction::IfElse { .. }
                    | Instruction::MakeArray { .. }
                    | Instruction::Noop => (),
                };
            }

            // If the function returns a reference it is impure
            let terminator = self.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values, .. }) = terminator {
                if return_values.iter().any(&contains_reference) {
                    return Purity::Impure;
                }
            }
        }

        result
    }
}

fn analyze_call_graph(
    call_graph: CallGraph,
    starting_purities: FunctionPurities,
) -> FunctionPurities {
    // Now we can analyze it: a function is only as pure as all of
    // its called functions
    let times_called = call_graph.times_called();
    let starting_points =
        times_called.iter().filter_map(|(id, times_called)| (*times_called == 0).then_some(*id));

    // The `starting_purities` are the preliminary results from `is_pure`
    // that don't take into account function calls. These finished purities do.
    let mut finished_purities = HashMap::default();

    let graph = call_graph.graph();
    let ids_to_indices = call_graph.ids_to_indices();
    let indices_to_ids = call_graph.indices_to_ids();

    for start_point in starting_points {
        let start_index = ids_to_indices[&start_point];
        let mut dfs = DfsPostOrder::new(graph, start_index);

        while let Some(index) = dfs.next(graph) {
            let id = indices_to_ids[&index];
            let mut purity = starting_purities[&id];

            for neighbor_index in graph.neighbors(index) {
                let neighbor = indices_to_ids[&neighbor_index];

                let neighbor_purity = finished_purities.get(&neighbor).copied().unwrap_or({
                    // The dependent function isn't finished yet. Since we're following
                    // calls in a DFS, this means there are mutually recursive functions.
                    // We could handle these but would need a different, much slower algorithm
                    // to detect strongly connected components. Instead, since this should be
                    // a rare case, we bail and assume impure for now.
                    if neighbor == id {
                        // If the recursive call is to the same function we can ignore it
                        purity
                    } else {
                        Purity::Impure
                    }
                });
                purity = purity.unify(neighbor_purity);
            }

            finished_purities.insert(id, purity);
        }
    }

    // Any remaining functions are completely unreachable and are either recursive or mutually recursive.
    // As these functions will be removed from the program, we treat them as impure.
    let unhandled_functions: Vec<_> =
        starting_purities.keys().filter(|func| !finished_purities.contains_key(*func)).collect();
    for id in unhandled_functions {
        finished_purities.insert(*id, Purity::Impure);
    }

    finished_purities
}
#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{ir::function::FunctionId, opt::pure::Purity, ssa_gen::Ssa},
    };

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
                jmpif v1 then: b1, else: b2
              b1():
                jmp b3(Field 0)
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

        let purities = &ssa.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(2)], Purity::Impure);
        assert_eq!(purities[&FunctionId::test_new(3)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(4)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(5)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(6)], Purity::Pure);
        assert_eq!(purities[&FunctionId::test_new(7)], Purity::Pure);

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
        acir(inline) pure fn pure_recursive f7 {
          b0(v0: u32):
            v3 = lt v0, u32 1
            jmpif v3 then: b1, else: b2
          b1():
            jmp b3(Field 0)
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
        brillig(inline) fn pure_with_predicate_func f3 {
          b0():
            return
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();

        let purities = &ssa.function_purities;
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

        let purities = &ssa.function_purities;
        assert_eq!(purities[&FunctionId::test_new(0)], Purity::PureWithPredicate);
        assert_eq!(purities[&FunctionId::test_new(1)], Purity::PureWithPredicate);
    }
}
