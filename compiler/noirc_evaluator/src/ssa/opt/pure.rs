use std::collections::BTreeSet;
use std::sync::Arc;

use fxhash::FxHashMap as HashMap;
use petgraph::prelude::DiGraph;
use petgraph::prelude::NodeIndex as PetGraphIndex;
use petgraph::visit::Dfs;

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
        let mut purities = HashMap::default();
        let mut called_functions = HashMap::default();

        // First look through each function to get a baseline on its purity and collect
        // the functions it calls to build a call graph.
        for function in self.functions.values() {
            let (purity, dependencies) = function.is_pure();
            purities.insert(function.id(), purity);
            called_functions.insert(function.id(), dependencies);
        }

        // Then transitively 'infect' any functions which call impure functions as also
        // impure.
        let purities = analyze_call_graph(called_functions, purities, self.main_id);
        let purities = Arc::new(purities);
        for (f, p) in purities.iter() {
            eprintln!("{f}: {p}");
        }

        // We're done, now store purities somewhere every dfg can find it.
        for function in self.functions.values_mut() {
            function.dfg.set_function_purities(purities.clone());
        }

        self
    }
}

pub(crate) type FunctionPurities = HashMap<FunctionId, Purity>;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Purity {
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
    fn is_pure(&self) -> (Purity, BTreeSet<FunctionId>) {
        let contains_reference = |value_id: &ValueId| {
            let typ = self.dfg.type_of_value(*value_id);
            typ.contains_reference()
        };

        if self.parameters().iter().any(&contains_reference) {
            return (Purity::Impure, BTreeSet::new());
        }

        // Set of functions we call which the purity result depends on.
        // `is_pure` is intended to be called on each function, building
        // up a call graph of sorts to check afterwards to propagate impurity
        // from called functions to their callers. Resultingly, an initial "Pure"
        // result here could be overridden by one of these dependencies being impure.
        let mut dependencies = BTreeSet::new();
        let mut result = Purity::Pure;

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
                    | Instruction::RangeCheck { .. } => {
                        result = Purity::PureWithPredicate;
                    }

                    // These instructions may be pure unless:
                    // - We may divide by zero
                    // - The array index is out of bounds.
                    // For both cases we can still treat them as pure if the arguments are known
                    // constants.
                    ins @ (Instruction::Binary(_)
                    | Instruction::ArrayGet { .. }
                    | Instruction::ArraySet { .. }) => {
                        if ins.requires_acir_gen_predicate(&self.dfg) {
                            result = Purity::PureWithPredicate;
                        }
                    }
                    Instruction::Call { func, .. } => {
                        match &self.dfg[*func] {
                            Value::Function(function_id) => {
                                // We don't know if this function is pure or not yet,
                                // so track it as a dependency for now.
                                dependencies.insert(*function_id);
                            }
                            Value::Intrinsic(intrinsic) => match intrinsic.purity() {
                                Purity::Pure => (),
                                Purity::PureWithPredicate => result = Purity::PureWithPredicate,
                                Purity::Impure => return (Purity::Impure, BTreeSet::new()),
                            },
                            // The function we're calling is unknown in the remaining cases,
                            // so just assume the worst.
                            Value::ForeignFunction(_)
                            | Value::Global(_)
                            | Value::Instruction { .. }
                            | Value::Param { .. }
                            | Value::NumericConstant { .. } => {
                                return (Purity::Impure, BTreeSet::new())
                            }
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
                }
            }

            // If the function returns a reference it is impure
            let terminator = self.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values, .. }) = terminator {
                if return_values.iter().any(&contains_reference) {
                    return (Purity::Impure, BTreeSet::new());
                }
            }
        }

        (result, dependencies)
    }
}

fn analyze_call_graph(
    dependencies: HashMap<FunctionId, BTreeSet<FunctionId>>,
    starting_purities: FunctionPurities,
    main: FunctionId,
) -> FunctionPurities {
    let (graph, ids_to_indices, indices_to_ids) = build_call_graph(dependencies);

    // Now we can analyze it: a function is only as pure as all of
    // its called functions
    let main_index = ids_to_indices[&main];
    let mut dfs = Dfs::new(&graph, main_index);

    // The `starting_purities` are the preliminary results from `is_pure`
    // that don't take into account function calls. These finished purities do.
    let mut finished_purities = HashMap::default();

    while let Some(index) = dfs.next(&graph) {
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
                Purity::Impure
            });
            purity = purity.unify(neighbor_purity);
        }

        finished_purities.insert(id, purity);
    }

    finished_purities
}

fn build_call_graph(
    dependencies: HashMap<FunctionId, BTreeSet<FunctionId>>,
) -> (DiGraph<FunctionId, ()>, HashMap<FunctionId, PetGraphIndex>, HashMap<PetGraphIndex, FunctionId>)
{
    let mut graph = DiGraph::new();
    let mut ids_to_indices = HashMap::default();
    let mut indices_to_ids = HashMap::default();

    for function in dependencies.keys() {
        let index = graph.add_node(*function);
        ids_to_indices.insert(*function, index);
        indices_to_ids.insert(index, *function);
    }

    // Create edges from caller -> called
    for (function, dependencies) in dependencies {
        let function_index = ids_to_indices[&function];

        for dependency in dependencies {
            let dependency_index = ids_to_indices[&dependency];
            graph.add_edge(function_index, dependency_index, ());
        }
    }

    (graph, ids_to_indices, indices_to_ids)
}
