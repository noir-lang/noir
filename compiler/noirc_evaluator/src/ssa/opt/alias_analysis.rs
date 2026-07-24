//! Steensgaard-style alias analysis for SSA references.
//!
//! This is a flow-insensitive, unification-based alias analysis. It walks every
//! instruction in no specific order and builds equivalence classes (alias sets) of
//! references that may point to the same memory location.
//!
//! On top of Steensgaard analysis, we also collect allocation sites and propagate their definition when possible.
//! This is done in a separate must_alias pass.
//!
//! The analysis is a pure read-only pass: it does not modify the IR. Other
//! passes can consume the result to make sound optimization decisions.
//!
//! # Preconditions
//! The analysis should be done after defunctionalization and lower ACIR references,
//! but it is not mandatory. If not, you will just lose precision.
//! The analysis assumes global values do not hold reference and will panic if they do.
//!
//! Supporting globals with reference would not be too difficult:
//! 1. unify shared 'globals having reference' across functions in the `GlobalValueId` type
//! 2. add them to the signature-based analysis.
//!
//! ## Algorithm outline
//!
//! Each reference-typed value is assigned into a single alias set via a union-find structure.
//! Instructions update the alias sets using the 4 constraints: a = b, a = &b, a = *b, *a = b
//!
//! The analysis is inter-procedural: arguments and parameters are unified for all call sites of a function
//! as well as the results and the function's return values.
//! To make the analysis order independent, the unified returned values of a function is stored (and updated after every call)
//! and initialized either by the results or the returned values (depending on which comes first)
//! Because `ValueIds` are per function, we have to reason instead on `GlobalValueId`: (`FunctionId`, `ValueId`).
//!
//! After processing all instructions, the union-find partitions every reference
//! into alias classes. Two references are *may-alias* if and only if they
//! belong to the same class, have the same type and have no distinct known allocation site.
//!
//! ## Additions/Changes to the standard algorithm
//!
//! ### Function pointers
//! Function pointers are not handled here, contrary to the textbook approach.
//! This is because defunctionalization is run early in the Ssa workflow, so it is probably
//! not worth the additional complexity
//!
//! ### Type-based filtering
//! May-alias queries use type information to recover some of the precision lost due to field-insensitivity
//!
//! ### Unresolved Calls
//! Unresolved function calls are not handled in Steensgaard analysis, however not all functions
//! are known in a Noir program (e.g Foreign calls). We add support for such unresolved calls
//! through a conservative analysis based on the function's signature. This allows us to support:
//! - high-order functions before defunctionalization
//! - foreign calls (although passing references to foreign calls is not allowed)
//! - single function analysis (although not recommended)
//!
//! This part is quadratic in the number of arguments/results, while the classic Steensgaard analysis is quasi-linear.
//!
//! ### Allocation sites
//!
//!
//! #### Must Alias
//! We can recover some precision by tracking allocation sites (i.e values that are the result of an Allocate instruction).
//! Two values sharing the same `Known(site)` only must-alias if that static
//! `Allocate` instruction fires at most once per program execution.
//! The allocation sites are computed in a separate `MustAliasAnalysis` pass, run at the end.
//! They are used for `must_alias` and `cannot_alias queries`.
//!
//! ## References
//!
//! Bjarne Steensgaard, "Points-to Analysis in Almost Linear Time", POPL 1996.
//! <https://dl.acm.org/doi/abs/10.1145/237721.237727>

use std::sync::Arc;

use iter_extended::vecmap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::hash_map::Entry;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        call_graph::CallGraph,
        cfg::ControlFlowGraph,
        function::{Function, FunctionId},
        instruction::{Instruction, Intrinsic, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        union_find::UnionFind,
        value::{Value, ValueId},
    },
    opt::must_alias::{AllocationLattice, MustAliasAnalysis},
    ssa_gen::Ssa,
};

/// `GlobalValueId` are `ValueId` along with their `FunctionId`,
/// allowing to globally use `ValueIds` coming from several functions.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct GlobalValueId(FunctionId, ValueId);

impl GlobalValueId {
    pub(crate) fn new(function: &Function, value: ValueId) -> Self {
        GlobalValueId(function.id(), value)
    }

    pub(crate) fn value_id(&self) -> ValueId {
        self.1
    }

    pub(crate) fn function_id(&self) -> FunctionId {
        self.0
    }
}

pub(crate) struct AliasAnalysis {
    /// union-find structure mapping `GlobalValueId` to their alias class.
    aliases: UnionFind<GlobalValueId>,

    /// Maps an alias class representative to the alias class of what it points to.
    points_to: HashMap<GlobalValueId, GlobalValueId>,

    /// Number of values in each alias class, keyed by the class representative.
    /// Populated lazily on the first `is_aliased` call so that `analyze` stays
    /// cheap for consumers that only query `may_alias`.
    class_sizes: Option<HashMap<GlobalValueId, u32>>,

    /// Known allocation sites: map a value to the `Allocate` that defined it.
    /// This is used to recover precision by saying that two values having
    /// two distinct allocation sites cannot alias, and computed during
    /// the must-alias analysis pass.
    allocation_sites: HashMap<GlobalValueId, AllocationLattice>,
}

impl AliasAnalysis {
    /// Run alias analysis on all `functions` and return the computed alias sets.
    ///
    /// The constraints are monotone and converge in a single pass.
    pub(crate) fn analyze(ssa: &Ssa) -> Self {
        let mut alias = AliasAnalysisContext::analyze_ssa(ssa);
        let must_alias = MustAliasAnalysis::analyze(ssa, &alias);
        alias.allocation_sites = must_alias.allocation_sites();
        alias
    }

    /// Build an analysis for one function in isolation. All calls are treated as
    /// opaque via `unresolved_call`. Less precise than [`Self::analyze`] but usable
    /// when the whole SSA is unavailable.
    // The single-function scope is exercised by unit tests; no production consumer
    // yet (intended for an early, pre-inlining mem2reg run).
    #[allow(dead_code)]
    pub(crate) fn analyze_single_function(function: &Function) -> Self {
        let mut alias = AliasAnalysisContext::analyze_single_function(function);
        let must_alias = MustAliasAnalysis::analyze_single_function(function, &alias);
        alias.allocation_sites = must_alias.allocation_sites();
        alias
    }

    /// Returns the representative of the alias class containing `value`.
    /// Two values share a class iff their `class_root` is equal.
    pub(crate) fn class_root(&self, value: GlobalValueId) -> GlobalValueId {
        self.aliases.find_immutable(value).unwrap_or(value)
    }

    /// Returns the alias class that `value`'s class points to, if any.
    /// Lets the must-alias pass walk reference chains (e.g. to poison the
    /// pointees reachable from an entry-point parameter).
    pub(crate) fn pointee(&self, value: GlobalValueId) -> Option<GlobalValueId> {
        self.points_to.get(&self.class_root(value)).copied()
    }

    /// Returns each alias-class representative mapped to its members.
    pub(crate) fn class_sets(&self) -> HashMap<GlobalValueId, Vec<GlobalValueId>> {
        self.aliases.class_sets()
    }

    /// Returns `true` if `a` and `b` may refer to the same memory location.
    ///
    /// Takes `&mut self` because of path compression.
    /// This has no visible side-effect and is perfectly safe.
    pub(crate) fn may_alias(
        &mut self,
        function: &Function,
        a: GlobalValueId,
        b: GlobalValueId,
    ) -> bool {
        if a == b {
            return true;
        }

        // Field-insensitivity may alias values with distinct types, but such values cannot alias.
        // Types are compared with [Type::canonical_eq] so `&T` and `&mut T` count as equal.
        // Note that this check is done only when both `a` and `b` match the given `function`.
        // This is purely for convenience, because the type filter would need access to the SSA
        // to look up types in other functions, which it doesn't currently.
        if function.id() == a.function_id() && a.function_id() == b.function_id() {
            let type_a = function.dfg.type_of_value(a.value_id());
            let type_b = function.dfg.type_of_value(b.value_id());
            if !type_a.canonical_eq(&type_b) {
                return false;
            }
        }

        if self.cannot_equal(a, b) {
            return false;
        }

        let a_root = self.aliases.find_existing(a);
        let b_root = self.aliases.find_existing(b);

        a_root == b_root
    }

    /// Returns `true` if `value` may be aliased with some other value in the program.
    ///
    /// The per-class size table is populated on demand the first time this is
    /// called — consumers that only use [`Self::may_alias`] do not pay for it.
    // Part of the analysis's query API, exercised by unit tests; no production
    // consumer yet.
    #[allow(dead_code)]
    pub(crate) fn is_aliased(&mut self, value: GlobalValueId) -> bool {
        let root = self.aliases.find_existing(value);
        if self.class_sizes.is_none() {
            // Count members per alias class
            self.class_sizes = Some(self.aliases.class_sizes());
        }
        let sizes = self.class_sizes.as_ref().expect("just populated");
        !matches!(sizes.get(&root), None | Some(1))
    }

    /// Recursively check if `target` can be referenced by `from`
    pub(crate) fn may_reference(&mut self, from: GlobalValueId, target: GlobalValueId) -> bool {
        let from_rep = self.aliases.find_existing(from);
        let target_rep = self.aliases.find_existing(target);
        if from_rep == target_rep {
            return !self.cannot_equal(from, target);
        }
        let mut seen = HashSet::default();
        let mut current = from_rep;
        while seen.insert(current) {
            match self.points_to.get(&current) {
                Some(&next) => {
                    let next = self.aliases.find(next);
                    if next == target_rep {
                        return true;
                    }
                    current = next;
                }
                None => return false,
            }
        }
        false
    }

    /// Returns `true` if `a` and `b` definitely refer to the same memory location
    /// Allocation site identity does not imply runtime cell identity when
    /// the site fires multiple times in one execution (e.g. loops, recursion)
    // Part of the analysis's query API, exercised by unit tests; no production
    // consumer yet.
    #[allow(dead_code)]
    pub(crate) fn must_alias(&self, a: GlobalValueId, b: GlobalValueId) -> bool {
        if a == b {
            return true;
        }
        let result = match (self.known_site(a), self.known_site(b)) {
            (Some(sa), Some(sb)) => sa == sb,
            _ => false,
        };
        if result {
            //Sanity check: must-alias values must be in the same alias class
            debug_assert_eq!(
                self.aliases.find_immutable(a).unwrap(),
                self.aliases.find_immutable(b).unwrap()
            );
        }
        result
    }

    pub(crate) fn cannot_equal(&self, a: GlobalValueId, b: GlobalValueId) -> bool {
        if a == b {
            return false;
        }
        let site_a = self.get_site(a);
        let site_b = self.get_site(b);
        site_a.cannot_equal(site_b)
    }

    /// Returns the known allocation site for `value`, if any.
    pub(crate) fn known_site(&self, value: GlobalValueId) -> Option<GlobalValueId> {
        match self.get_site(value) {
            AllocationLattice::Known(site) => Some(site),
            _ => None,
        }
    }

    /// Read `value_sites[value]`, defaulting to `Undef` if absent.
    pub(crate) fn get_site(&self, value: GlobalValueId) -> AllocationLattice {
        self.allocation_sites.get(&value).copied().unwrap_or(AllocationLattice::Undef)
    }
}

/// Merge rule to apply to the bucket representatives at an
/// `unresolved_call`. Indices reference positions in the call's
/// canonically-ordered `Vec<Type>` / `Vec<GlobalValueId>`.
#[derive(PartialEq, Eq, Clone, Copy)]
enum SignatureTemplate {
    MergeAlias(usize, usize),
    MergeReference(usize, usize),
}

/// `AliasAnalysis` stores the result of the alias analysis pass
/// as well as transient data computed during the analysis
#[derive(Default)]
struct AliasAnalysisContext {
    /// union-find structure mapping `GlobalValueId` to their alias class.
    aliases: UnionFind<GlobalValueId>,

    /// Maps an alias class representative to the alias class of what it points to.
    points_to: HashMap<GlobalValueId, GlobalValueId>,

    /// Class representative of values returned by the called functions.
    /// This is used during the analysis and is discarded afterwards.
    return_values: HashMap<FunctionId, Vec<GlobalValueId>>,

    /// Map a type with the reference types recursively contained into it.
    /// Used (and computed) during analysis of unresolved function calls.
    /// `Arc<HashSet<_>>` is used to avoid borrow checker issues
    reference_types: HashMap<Type, Arc<HashSet<Type>>>,

    /// Signature Templates: cache template rules for a canonicalized signature
    /// represented by a vector of (distinct and ordered) Types
    signatures: HashMap<Vec<Type>, Vec<SignatureTemplate>>,
}

impl AliasAnalysisContext {
    fn analyze_ssa(ssa: &Ssa) -> AliasAnalysis {
        let mut analysis = Self::default();

        // The analysis tolerates incomplete call-graphs, via `unresolved_call` handling
        let call_graph = CallGraph::from_ssa_partial(ssa);
        let (sccs, _) = call_graph.sccs();
        // Process functions in calling order. Any order works.
        let functions: Vec<&Function> =
            sccs.into_iter().rev().flatten().filter_map(|fid| ssa.functions.get(&fid)).collect();

        if let Some(first) = functions.first() {
            Self::assert_globals_without_references(first);
        }

        for function in &functions {
            analysis.analyze_function(Some(ssa), function);
        }

        analysis.into_result()
    }

    fn analyze_single_function(function: &Function) -> AliasAnalysis {
        Self::assert_globals_without_references(function);
        let mut analysis = Self::default();
        analysis.analyze_function(None, function);
        analysis.into_result()
    }

    /// Globals are expected to be pure constants (numeric or composite-of-numeric)
    /// as documented. The global table is shared across functions, so checking any
    /// one function's globals covers the whole analysis.
    fn assert_globals_without_references(function: &Function) {
        for (_, global) in function.dfg.globals.values_iter() {
            assert!(
                !global.get_type().contains_reference(),
                "ICE: alias_analysis assumes globals do not have references"
            );
        }
    }

    /// Finalize the accumulated union-find into the queryable [`AliasAnalysis`].
    fn into_result(self) -> AliasAnalysis {
        AliasAnalysis {
            aliases: self.aliases,
            points_to: self.points_to,
            class_sizes: None,
            allocation_sites: HashMap::default(),
        }
    }

    /// Walk every block in one function, processing instructions and terminators.
    /// If the function is an entry point of the SSA, also unify its
    /// same-typed reference parameters.
    fn analyze_function(&mut self, ssa: Option<&Ssa>, function: &Function) {
        let is_entry_point = match ssa {
            Some(ssa) => ssa.is_entry_point(function.id()),
            None => true,
        };

        if is_entry_point {
            // Unify the reference parameters of the entry point because the
            // external caller may pass the same reference to 2 reference parameters.
            let params = function.dfg[function.entry_block()].parameters();
            self.unresolved_call(function, params, &[]);
        }

        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_cfg(&cfg);
        let blocks = post_order.into_vec_reverse();

        for block_id in blocks {
            self.analyze_block(function, block_id, ssa);
        }

        // Make the analysis total: insert every reference-typed value into the
        // union-find (most are added while processing constraints, but values
        // never involved in one would otherwise be missing). With the structure
        // total, a post-analysis lookup of an unknown value is a bug —
        // see `find_existing` — rather than a silently-created singleton.
        for (value_id, _) in function.dfg.values_iter() {
            if function.dfg.type_of_value(value_id).contains_reference() {
                self.aliases.make_set(GlobalValueId::new(function, value_id));
            }
        }
    }

    // Returns the set of values that `pointer` may point to.
    // Uninitialized pointers (or none reference values) will return `None`.
    // self is mutable because of path compression in the union-find.
    fn get_pointee(&mut self, pointer: GlobalValueId) -> Option<GlobalValueId> {
        self.points_to.get(&self.aliases.find(pointer)).copied()
    }

    fn set_pointee(&mut self, pointer: GlobalValueId, target: GlobalValueId) {
        self.points_to.insert(self.aliases.find(pointer), target);
    }

    /// Process all instructions in a single block, updating the alias sets.
    /// `ignore_allocations_in_block` is  used to mark every `Allocate` defined in
    /// this block as `loop_allocate`, since the loop may fire it many times per invocation.
    fn analyze_block(&mut self, function: &Function, block_id: BasicBlockId, ssa: Option<&Ssa>) {
        let block = &function.dfg[block_id];

        for instruction_id in block.instructions() {
            let results = function.dfg.instruction_results(*instruction_id);
            match &function.dfg[*instruction_id] {
                Instruction::Allocate => {
                    // Defines a new pointer value.
                    let address = GlobalValueId::new(function, results[0]);
                    self.aliases.make_set(address);
                }
                Instruction::Load { address } => {
                    // Complex constraint type 1: result = *address
                    self.merge_reference(function, results[0], *address);
                }
                Instruction::Store { address, value } => {
                    // Complex constraint type 2: *address = value
                    self.merge_reference(function, *value, *address);
                }
                Instruction::Call { func: callee_id, arguments } => {
                    match &function.dfg[*callee_id] {
                        // Inter-procedural analysis for resolved functions
                        // - merge arguments with their parameters,
                        // - process the function body (i.e analyze the instructions, but only once since it context-insensitive).
                        //   This is done through analyze_function() which process all the functions.
                        // - merge return values with the instruction results
                        Value::Function(callee_id) if ssa.is_some() => {
                            self.unify_call_arguments_and_return(
                                ssa.unwrap(),
                                *callee_id,
                                function,
                                arguments,
                                results,
                            );
                        }
                        Value::Intrinsic(Intrinsic::Hint(_)) => {
                            self.unresolved_call(function, arguments, results);
                        }
                        Value::Intrinsic(intrinsic) if Self::is_vector_intrinsic(intrinsic) => {
                            // Merge input vector with output,
                            // Add the elements to the vector's pointee set
                            self.unify_vector_intrinsic(function, intrinsic, arguments, results);
                        }
                        Value::Intrinsic(intrinsic) => {
                            // Only Hint or Vector operations may alias.
                            assert!(!Self::intrinsic_may_alias(intrinsic));
                        }
                        // Foreign calls cannot receive or return references in
                        // Noir, so they cannot create or transport aliases —
                        // handle them optimistically (no effect). Only genuinely
                        // unresolved callees (the fallthrough) are conservative.
                        Value::ForeignFunction { .. } => {}
                        // Fallthrough for unresolved functions whose function body
                        // is not available, via a conservative type-based analysis.
                        _ => {
                            self.unresolved_call(function, arguments, results);
                        }
                    }
                }
                Instruction::ArrayGet { array, .. } => {
                    // Field-insensitive: array's pointee is merged with the elements (and the index is not used)
                    // Note that composite arrays can hold different types, which obviously cannot alias.
                    // So merging the elements is sound but imprecise.
                    self.merge_reference(function, results[0], *array);
                }
                Instruction::ArraySet { array, value, .. } => {
                    let new_array = function.dfg.instruction_result::<1>(*instruction_id)[0];

                    // Field-insensitive: the array is merged with its elements (and the index is not used)
                    self.merge_reference(function, *value, new_array);
                    // The original array is merged with the new one transitively through the value, because they share elements:
                    // Ex: a=[e1, e2]; a1 = ArraySet(a, 0 , v);
                    // a[1] and a1[1] are the same. Because of field-insensitivity, a[1] aliases with a and a1[1] aliases with a1,
                    // so a and a1 aliases.
                    self.merge_reference(function, *value, *array);
                }
                Instruction::MakeArray { elements, .. } => {
                    let array = results[0];
                    for element in elements {
                        // Field-insensitive: each element joins new_array's pointee class
                        self.merge_reference(function, *element, array);
                    }
                }
                Instruction::IfElse { then_value, else_value, .. } => {
                    let typ = function.dfg.type_of_value(*then_value);
                    if typ.contains_reference() {
                        let result_g = GlobalValueId::new(function, results[0]);
                        let then_g = GlobalValueId::new(function, *then_value);
                        let else_g = GlobalValueId::new(function, *else_value);
                        self.merge_alias(then_g, result_g);
                        self.merge_alias(else_g, result_g);
                    }
                }
                // All other instructions have no alias effects.
                _ => {}
            }
        }

        self.analyze_terminator(function, block.terminator());
    }

    // Base constraint a = &b: merge via recursive union b with the pointee of a
    // Do nothing if b is not a reference so that the aliases are not polluted by non-reference values.
    fn merge_reference(&mut self, function: &Function, b: ValueId, a: ValueId) {
        if function.dfg.type_of_value(b).contains_reference() {
            let b = GlobalValueId::new(function, b);
            let a = GlobalValueId::new(function, a);
            if let Some(values) = self.get_pointee(a) {
                self.merge_alias(b, values);
            } else {
                // Lazy initialization of the pointer
                self.set_pointee(a, b);
            }
        }
    }

    /// Simple constraint `a = b`: merge a and b if they carry references.
    /// Scalars are ignored to avoid polluting the union-find, by checking the
    /// type of `a`, which is supposed to belong to `function`.
    fn simple_constraint(&mut self, function: &Function, a: ValueId, b: GlobalValueId) {
        if function.dfg.type_of_value(a).contains_reference() {
            let a = GlobalValueId::new(function, a);
            self.merge_alias(a, b);
        }
    }

    // Merge alias classes recursively
    fn merge_alias(&mut self, a: GlobalValueId, b: GlobalValueId) {
        let root_a = self.aliases.find(a);
        let root_b = self.aliases.find(b);
        if root_a == root_b {
            // Already the same class, nothing to do.
            return;
        }
        self.aliases.union(root_a, root_b);

        // Recursively merge the points_to, if they exist.
        let root = self.aliases.find(root_a);
        let points_a = self.points_to.get(&root_a);
        let points_b = self.points_to.get(&root_b);
        match (points_a, points_b) {
            (Some(&pa), Some(&pb)) => {
                self.points_to.insert(root, pa);
                self.merge_alias(pa, pb);
            }
            (Some(&pa), None) => {
                self.points_to.insert(root, pa);
            }
            (None, Some(&pb)) => {
                self.points_to.insert(root, pb);
            }
            (None, None) => {} // Nothing to do, root points_to will be lazily initialized when it is used.
        }
    }

    /// Process a terminator instruction.
    fn analyze_terminator(
        &mut self,
        function: &Function,
        terminator: Option<&TerminatorInstruction>,
    ) {
        let Some(terminator) = terminator else {
            return;
        };
        match terminator {
            TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            } => {
                self.unify_with_block_params(function, *then_destination, then_arguments);
                self.unify_with_block_params(function, *else_destination, else_arguments);
            }
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                self.unify_with_block_params(function, *destination, arguments);
            }
            TerminatorInstruction::Return { return_values, .. } => {
                self.merge_return(function, return_values);
            }
            TerminatorInstruction::Unreachable { .. } => (),
        }
    }

    /// At each Return, merge the returned values with the function's canonical
    /// `return_values` (or create them the first time).
    fn merge_return(&mut self, function: &Function, return_values: &[ValueId]) {
        match self.return_values.get(&function.id()).cloned() {
            Some(results) => {
                debug_assert_eq!(return_values.len(), results.len());
                for (value, result) in return_values.iter().zip(results.iter()) {
                    self.simple_constraint(function, *value, *result);
                }
            }
            None => {
                let return_values = return_values.iter().map(|v| GlobalValueId::new(function, *v));
                self.return_values.insert(function.id(), return_values.collect());
            }
        }
    }

    fn unify_with_block_params(
        &mut self,
        function: &Function,
        destination: BasicBlockId,
        arguments: &[ValueId],
    ) {
        let params = function.dfg[destination].parameters();
        debug_assert_eq!(arguments.len(), params.len());
        for (arg, param) in arguments.iter().zip(params.iter()) {
            if function.dfg.type_of_value(*arg).contains_reference() {
                // Merge argument and its parameter (refs and composites carrying refs).
                let arg = GlobalValueId::new(function, *arg);
                let param = GlobalValueId::new(function, *param);
                self.merge_alias(arg, param);
            }
        }
    }

    /// Conservative analysis of unresolved call
    ///
    /// For each pair of ref-carrying args/results, link them according to
    /// their type relationship:
    /// 1. Same type => `merge_alias`: they might be the same value.
    /// 2. `outer` can contain `inner`'s type => `merge_reference(inner, outer)`:
    ///    `inner` could be a pointee/element of `outer`.
    /// 3. Any other case where a nested reference type of one could reach a
    ///    structural nested type of the other => `merge_alias`: further
    ///    extractions on either side may land in the same alias class.
    ///    Ex:   `a: [{Field, &Field}; 2]` and `b: &{Field, int}` => &T in a, and T in b (T=Field)
    ///
    /// Note that this is quadratic in the number of arguments/results
    /// because we need to analyze every pair. Instead we aggregate identical types
    /// and sort them to get a canonical signature, and pre-compute a 'template'
    /// only once for each signature. Only this computation per signature is quadratic in
    /// the number of distinct reference types.
    fn unresolved_call(&mut self, function: &Function, arguments: &[ValueId], results: &[ValueId]) {
        // Collect each ref-carrying value along with its type and a cached
        // `Arc<HashSet<Type>>` of all reference types it contains. The cache
        // is shared across calls, and computed once.
        let entries: Vec<(ValueId, Type, Arc<HashSet<Type>>)> = arguments
            .iter()
            .chain(results.iter())
            .copied()
            .filter_map(|v| {
                let typ = function.dfg.type_of_value(v).into_owned();
                if !typ.contains_reference() {
                    return None;
                }
                // Canonicalize so `&T` and `&mut T` have the cache entries
                let typ = typ.canonicalized();
                let refs = self.get_ref_types(&typ);
                Some((v, typ, refs))
            })
            .collect();
        // Merge identical types into buckets
        let (types, representatives) = self.type_representatives(function, entries);
        // Retrieve, or build the template corresponding to the signature
        let templates = self.build_signature(types);
        // Apply the template
        for template in templates {
            match template {
                SignatureTemplate::MergeAlias(a, b) => {
                    self.merge_alias(representatives[a], representatives[b]);
                }
                SignatureTemplate::MergeReference(pointed, pointer) => {
                    self.merge_reference(
                        function,
                        representatives[pointed].1,
                        representatives[pointer].1,
                    );
                }
            }
        }
    }

    fn build_signature(&mut self, bucket: Vec<Type>) -> Vec<SignatureTemplate> {
        let signature = self.signatures.get(&bucket);
        if let Some(signature) = signature {
            return signature.clone();
        }
        let mut templates = Vec::new();
        for i in 0..bucket.len() {
            let type_a = &bucket[i];
            for (j, type_b) in bucket.iter().enumerate().skip(i + 1) {
                if Self::type_can_contain(type_a, type_b) {
                    // Case 2: b could be an element/pointee of a.
                    templates.push(SignatureTemplate::MergeReference(j, i));
                } else if Self::type_can_contain(type_b, type_a) {
                    // Case 2 (other direction): a could be an element/pointee of b.
                    templates.push(SignatureTemplate::MergeReference(i, j));
                } else {
                    let refs_a = self.get_ref_types(type_a);
                    let refs_b = self.get_ref_types(type_b);
                    if Self::could_have_sub_ref_aliasing(&refs_a, &refs_b) {
                        // Case 3: a reference sub-type of one may reach a
                        // structural sub-type of the other (including the case
                        // where they simply share an inner ref type, since
                        // `T` structurally contains `T`).
                        templates.push(SignatureTemplate::MergeAlias(i, j));
                    }
                }
            }
        }
        self.signatures.insert(bucket, templates.clone());
        templates
    }

    /// Helper function for unresolved call which put identical types into buckets and merge their corresponding `ValueId`.
    /// Returns the canonicalized signature (one 'reference' type per bucket, sorted) and its corresponding vector of bucket representatives.
    fn type_representatives(
        &mut self,
        function: &Function,
        entries: Vec<(ValueId, Type, Arc<HashSet<Type>>)>,
    ) -> (Vec<Type>, Vec<GlobalValueId>) {
        let mut buckets: HashMap<Type, GlobalValueId> = HashMap::default();

        // An entry is either added to a new bucket or merged into the existing one.
        for (v, typ, _) in entries {
            let v_g = GlobalValueId::new(function, v);
            match buckets.entry(typ) {
                Entry::Occupied(e) => {
                    // Case 1: same type
                    let rep_g = *e.get();
                    self.merge_alias(rep_g, v_g);
                }
                Entry::Vacant(e) => {
                    e.insert(v_g);
                }
            }
        }
        // The buckets are sorted by type, so that we have only one signature even if the order change.
        let mut buckets: Vec<(Type, GlobalValueId)> = buckets.into_iter().collect();
        buckets.sort_by(|(a, _), (b, _)| a.cmp(b));
        buckets.into_iter().unzip()
    }

    /// True if `outer`'s shape permit `inner` as a possible pointee / element
    /// at any depth?
    /// The function is recursive but is ensured to terminate because Noir Types are non-recursive.
    ///
    /// Examples (all return true):
    /// - `&T` contains `T` (one level of pointer indirection).
    /// - `[T; N]` / `[T]` contains `T` as an element.
    /// - `&&T` contains `&T` (which transitively contains `T`).
    /// - `[&T; N]` contains `&T`, which transitively contains `T`.
    fn type_can_contain(outer: &Type, inner: &Type) -> bool {
        match outer {
            Type::Reference(t, _) => t.as_ref() == inner || Self::type_can_contain(t, inner),
            Type::Array(composite, _) | Type::Vector(composite) => {
                composite.iter().any(|slot| slot == inner || Self::type_can_contain(slot, inner))
            }
            _ => false,
        }
    }

    /// Collect every `Type::Reference(_)` appearing at any depth of `typ`.
    /// The result is cached per value and used in Case 3 of `unresolved_call`.
    fn get_ref_types(&mut self, typ: &Type) -> Arc<HashSet<Type>> {
        if let Some(cached) = self.reference_types.get(typ) {
            return Arc::clone(cached);
        }
        let mut refs = HashSet::default();
        match typ {
            Type::Reference(inner, _) => {
                refs.insert(typ.clone());
                // Recurse through the cache so `inner` gets cached too.
                let inner_refs = self.get_ref_types(inner);
                refs.extend(inner_refs.iter().cloned());
            }
            Type::Array(composite, _) | Type::Vector(composite) => {
                // Each composite slot goes through the cache individually.
                let composite = composite.clone();
                for slot in composite.iter() {
                    let slot_refs = self.get_ref_types(slot);
                    refs.extend(slot_refs.iter().cloned());
                }
            }
            _ => {}
        }
        let arc = Arc::new(refs);
        self.reference_types.insert(typ.clone(), Arc::clone(&arc));
        arc
    }

    /// Detects the sub-reference pattern: one value has a `&T` somewhere in
    /// its type, and the other has a `&C` somewhere where `T` appears as a
    /// structural (non-reference) element of `C`.
    ///
    /// E.g.: `a: [{Field, &Field}; N]` and `b: &[{Field, int}; M]`
    /// when a function f(a,b) does `a[j].1 = b[i].0`, a and b alias in a subtle way.
    fn could_have_sub_ref_aliasing(refs_a: &HashSet<Type>, refs_b: &HashSet<Type>) -> bool {
        for ref_a in refs_a {
            let Type::Reference(inner_a, _) = ref_a else { continue };
            for ref_b in refs_b {
                let Type::Reference(inner_b, _) = ref_b else { continue };
                if Self::type_contains_structurally(inner_a, inner_b)
                    || Self::type_contains_structurally(inner_b, inner_a)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Does `outer` structurally contain `inner` (at any depth, following
    /// only composite types — not references)? Returns true if `outer` is
    /// `inner`, or if `inner` appears as a slot of `outer`'s composite
    /// structure (transitively).
    ///
    /// E.g: does the following types contains structurally Field?
    /// - &{Field, int}: No, it's a reference
    /// - [Field; 3]: Yes, the array has a Field as element type.
    fn type_contains_structurally(outer: &Type, inner: &Type) -> bool {
        if outer == inner {
            return true;
        }
        match outer {
            Type::Array(composite, _) | Type::Vector(composite) => {
                composite.iter().any(|slot| Self::type_contains_structurally(slot, inner))
            }
            _ => false,
        }
    }

    /// True for intrinsics that manipulate a vector/array container.
    fn is_vector_intrinsic(intrinsic: &Intrinsic) -> bool {
        matches!(
            intrinsic,
            Intrinsic::AsVector
                | Intrinsic::VectorPushBack
                | Intrinsic::VectorPushFront
                | Intrinsic::VectorPopBack
                | Intrinsic::VectorPopFront
                | Intrinsic::VectorInsert
                | Intrinsic::VectorRemove
        )
    }

    /// Precise handling for vector-manipulating intrinsics: dispatches
    /// per-intrinsic based on the known argument/result layout
    ///
    /// - `AsVector`:        `(arr) -> (len, vec)`
    /// - `VectorPushBack`:  `(len, vec, elem) -> (new_len, new_vec)`
    /// - `VectorPushFront`: `(len, vec, elem) -> (new_len, new_vec)`
    /// - `VectorInsert`:    `(len, vec, idx, elem) -> (new_len, new_vec)`
    /// - `VectorPopBack`:   `(len, vec) -> (new_len, new_vec, elem)`
    /// - `VectorRemove`:    `(len, vec, idx) -> (new_len, new_vec, elem)`
    /// - `VectorPopFront`:  `(len, vec) -> (elem, new_len, new_vec)`
    fn unify_vector_intrinsic(
        &mut self,
        function: &Function,
        intrinsic: &Intrinsic,
        arguments: &[ValueId],
        results: &[ValueId],
    ) {
        match intrinsic {
            Intrinsic::AsVector => {
                self.unify_vector_op(function, arguments[0], results[1], &[]);
            }
            Intrinsic::VectorPushBack | Intrinsic::VectorPushFront => {
                let elements = &arguments[2..];
                self.unify_vector_op(function, arguments[1], results[1], elements);
            }
            Intrinsic::VectorInsert => {
                let elements = &arguments[3..];
                self.unify_vector_op(function, arguments[1], results[1], elements);
            }
            Intrinsic::VectorPopBack | Intrinsic::VectorRemove => {
                self.unify_vector_op(function, arguments[1], results[1], &results[2..]);
            }
            Intrinsic::VectorPopFront => {
                let n = results.len();
                let elements = &results[..n - 2];
                let output = results[n - 1];
                self.unify_vector_op(function, arguments[1], output, elements);
            }
            _ => unreachable!("non-vector intrinsic passed to unify_vector_intrinsic"),
        }
    }

    /// Common helper for vector intrinsics.
    ///
    /// - `input_vec` / `output_vec` are merged because they have common elements so they alias.
    /// - `elements`: are merged with their container's pointee (field insensitive)
    ///
    /// Early-returns if the input container carries no refs
    /// such intrinsics have no effect on the alias analysis.
    fn unify_vector_op(
        &mut self,
        function: &Function,
        input_vec: ValueId,
        output_vec: ValueId,
        elements: &[ValueId],
    ) {
        if !function.dfg.type_of_value(input_vec).contains_reference() {
            return;
        }
        let input = GlobalValueId::new(function, input_vec);
        let output = GlobalValueId::new(function, output_vec);
        self.merge_alias(input, output);
        for element in elements {
            self.merge_reference(function, *element, input_vec);
        }
    }

    /// Unify call arguments with callee's formal parameters, and call results with
    /// callee's canonical return reps. This is the Simple constraint applied
    /// at the call boundary.
    fn unify_call_arguments_and_return(
        &mut self,
        ssa: &Ssa,
        callee_id: FunctionId,
        caller: &Function,
        arguments: &[ValueId],
        results: &[ValueId],
    ) {
        let callee = &ssa.functions[&callee_id];

        // Function parameters are the entry block's parameters.
        let parameters = callee.dfg[callee.entry_block()].parameters();
        debug_assert_eq!(arguments.len(), parameters.len());
        for (arg, param) in arguments.iter().zip(parameters.iter()) {
            // Simple constraint: 'param = arg'
            let arg = GlobalValueId::new(caller, *arg);
            self.simple_constraint(callee, *param, arg);
        }

        // Return values: unify each call result with the canonical return_values.
        // If it does not exist yet, because the function has not been analyzed yet,
        // we simply initialize it with the call result.
        if let Some(rets) = self.return_values.get(&callee_id).cloned() {
            debug_assert_eq!(results.len(), rets.len());
            for (result, ret) in results.iter().zip(rets.iter()) {
                // Simple constraint: 'result = ret'
                self.simple_constraint(caller, *result, *ret);
            }
        } else {
            // Record results as the initial class representative. They'll be unified
            // with real return values when the function will be analyzed.
            let reps = vecmap(results, |v| GlobalValueId::new(caller, *v));
            self.return_values.insert(callee_id, reps);
        }
    }

    fn intrinsic_may_alias(intrinsic: &Intrinsic) -> bool {
        match intrinsic {
            // Intrinsics which cannot alias anything
            Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::StaticAssert
            | Intrinsic::ApplyRangeConstraint
            | Intrinsic::StrAsBytes
            | Intrinsic::ToBits(_)
            | Intrinsic::ToRadix(_)
            | Intrinsic::AsWitness
            | Intrinsic::DerivePedersenGenerators
            | Intrinsic::IsUnconstrained
            | Intrinsic::FieldLessThan
            | Intrinsic::ArrayRefCount
            | Intrinsic::VectorRefCount
            | Intrinsic::AssertConstant
            // BlackBox are dedicated to ACIR and are pure numerical computations.
            // They cannot alias, and will likely never alias.
            // But it should be verified for any new BlackBox though.
            | Intrinsic::BlackBox(_) => false,

            Intrinsic::AsVector
            | Intrinsic::VectorPushBack
            | Intrinsic::VectorPushFront
            | Intrinsic::VectorPopBack
            | Intrinsic::VectorPopFront
            | Intrinsic::VectorInsert
            | Intrinsic::VectorRemove
            | Intrinsic::Hint(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the alias analysis.
    use super::*;
    use crate::ssa::{ir::instruction::Instruction, ssa_gen::Ssa};

    /// Collect the result `ValueIds` of every `Allocate` instruction in the main
    /// function, in declaration order (across reachable blocks).
    fn collect_allocates(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Allocate) {
                    let id =
                        GlobalValueId::new(func, func.dfg.instruction_result::<1>(*inst_id)[0]);
                    out.push(id);
                }
            }
        }
        out
    }

    /// Collect the result `ValueIds` of every `Load` instruction.
    fn collect_loads(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Load { .. }) {
                    let id =
                        GlobalValueId::new(func, func.dfg.instruction_result::<1>(*inst_id)[0]);
                    out.push(id);
                }
            }
        }
        out
    }

    /// Collect the result `ValueIds` of every `ArrayGet` instruction.
    fn collect_array_gets(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::ArrayGet { .. }) {
                    let id =
                        GlobalValueId::new(func, func.dfg.instruction_result::<1>(*inst_id)[0]);
                    out.push(id);
                }
            }
        }
        out
    }

    fn collect_call_results_in_main(ssa: &Ssa) -> Vec<GlobalValueId> {
        let main = ssa.main();
        let mut out = Vec::new();
        for block in main.reachable_blocks() {
            for inst_id in main.dfg[block].instructions() {
                if matches!(main.dfg[*inst_id], Instruction::Call { .. }) {
                    for result in main.dfg.instruction_results(*inst_id) {
                        let id = GlobalValueId::new(main, *result);
                        out.push(id);
                    }
                }
            }
        }
        out
    }

    fn analyze_main(ssa: &Ssa) -> AliasAnalysis {
        AliasAnalysis::analyze(ssa)
    }

    // ============================================================
    // Basic cases
    // ============================================================

    #[test]
    fn two_distinct_allocates_do_not_alias() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
        // Neither ref was involved in a merge → both singletons → not aliased.
        assert!(!analysis.is_aliased(allocs[0]));
        assert!(!analysis.is_aliased(allocs[1]));
    }

    /// `&T` and `&mut T` can legitimately point to the same memory at runtime.
    /// Types are canonicalized throughout the analysis so the `may_alias` type guard
    /// treats them as equivalent.
    #[test]
    fn mutable_and_immutable_refs_are_treated_as_same_type() {
        // Use oracles so v0 and v1 carry to avoid allocation-site.
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_mut() -> &mut Field
            v1 = call oracle_ref() -> &Field
            call v10(v0, v1)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        // The opaque `print` call canonicalizes both arg types to `&Field`,
        // so Case 1 of `unresolved_call` fires and merges v0 with v1.
        // `may_alias` canonicalizes before the type guard → returns true.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    #[test]
    fn same_reference_aliases_itself() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), allocs[0], allocs[0]));
    }

    #[test]
    fn load_aliases_stored_value() {
        // Store v1 at *v0 then v2 = *v0. v2 should alias v1.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut &mut Field
            v1 = allocate -> &mut Field
            store v1 at v0
            v2 = load v0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), allocs[1], loads[0]));
    }

    #[test]
    fn ifelse_reference_branches_alias() {
        // IfElse on two references: both branches + result are in the same class.
        // v1 and v2 come from distinct oracles so they start in separate classes
        // (avoid the allocation-site refinement).
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = call oracle_a() -> &mut Field
            v2 = call oracle_b() -> &mut Field
            v3 = not v0
            v4 = if v0 then v1 else (if v3) v2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    #[test]
    fn make_array_unifies_element_classes() {
        // Field-insensitive: all refs placed into an array end up aliased.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field; 2]
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
        // MakeArray merged v0 and v1 into the array's pointee class → both aliased.
        assert!(analysis.is_aliased(call_results[0]));
        assert!(analysis.is_aliased(call_results[1]));
    }

    #[test]
    fn array_get_aliases_source_elements() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field; 2]
            v3 = array_get v2, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[0]));
        assert!(analysis.may_alias(ssa.main(), allocs[1], gets[0]));
    }

    #[test]
    fn array_set_preserves_aliasing_with_original() {
        // `new = array_set(arr, _, v)`: new and arr share elements (field-insensitive).
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field; 2]
            v3 = allocate -> &mut Field
            v4 = array_set v2, index u32 0, value v3
            v5 = array_get v4, index u32 1 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), allocs[1], gets[0]));
    }

    // ============================================================
    // Call handling
    // ============================================================

    #[test]
    fn inter_procedural_analysis() {
        // Two refs passed to the same call that does not alias them.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            call f1(v0, v1)
            return
        }
        acir(inline) fn f1 f1 {
          b0(v0: &mut Field, v1: &mut Field):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
    }

    #[test]
    fn different_call_sites_cross_contaminate() {
        // v0 and v1 aliases because they are results of the same function (although different call site)
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1() -> &mut Field
                v1 = call f1() -> &mut Field
                return
            }
            acir(inline) fn f1 f1 {
            b0():
                v0 = allocate -> &mut Field
                return v0
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    // ============================================================
    // Field-insensitivity
    // ============================================================

    #[test]
    fn struct_fields_are_merged_field_insensitively() {
        // Struct = Array with length=1 and multiple slots.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0, v1] : [(&mut Field, &mut Field); 1]
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    #[test]
    fn distinct_allocates_in_struct_do_not_alias() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [(&mut Field, &mut Field); 1]
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
    }

    // ============================================================
    // Loops / back-edges.
    // ============================================================

    #[test]
    fn loop_block_param_propagation() {
        // Block-param arg passing: v0 and the block param v1 should alias.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1(v0)
          b1(v1: &mut Field):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let v0 = collect_allocates(&ssa)[0];
        // Pick b1 — the second reachable block.
        let b1 = func.reachable_blocks().into_iter().nth(1).unwrap();
        let v1 = GlobalValueId::new(func, func.dfg[b1].parameters()[0]);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(func, v0, v1));
    }

    // ============================================================
    // OTHER CASES - more complex cases
    // ============================================================

    /// `IfElse` on composites merges the composite branches so
    /// refs extracted from the result alias both branches' underlying refs.
    #[test]
    fn ifelse_composite_extractions_alias() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            v3 = make_array [v1] : [&mut Field; 1]
            v4 = make_array [v2] : [&mut Field; 1]
            v5 = not v0
            v6 = if v0 then v3 else (if v5) v4
            v7 = array_get v6, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v6 extracted from v5 (= v3 or v4) should alias v1 and v2.
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[0]));
    }

    /// The caller should see the alias between a value extracted before a call
    /// and a value loaded after the call.
    #[test]
    fn callee_links_array_element_with_ref_argument() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            v2 = array_get v1, index u32 0 -> &mut Field
            v3 = allocate -> &mut &mut Field
            call f1(v1, v3)
            v4 = load v3 -> &mut Field
            return
        }
        acir(inline) fn f1 f1 {
          b0(v0: [&mut Field; 1], v1: &mut &mut Field):
            v2 = array_get v0, index u32 0 -> &mut Field
            store v2 at v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let gets = collect_array_gets(&ssa); // [main's v2, f1's v2]
        let loads = collect_loads(&ssa); // [main's v4]
        let mut analysis = analyze_main(&ssa);
        // main's v2 (extracted before call) should alias main's v4 (loaded after call).
        assert!(analysis.may_alias(ssa.main(), gets[0], loads[0]));
    }

    /// Call through nested composites: the callee extracts the
    /// deeply-nested ref from its array argument and stores it through its
    /// ref-to-ref argument. The caller's deep-extracted ref (done before the
    /// call) should then alias the value loaded from the ref-to-ref after it.
    #[test]
    fn callee_links_nested_array_element_with_ref_argument() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            v2 = make_array [v1] : [[&mut Field; 1]; 1]
            v3 = array_get v2, index u32 0 -> [&mut Field; 1]
            v4 = array_get v3, index u32 0 -> &mut Field
            v5 = allocate -> &mut &mut Field
            call f1(v2, v5)
            v6 = load v5 -> &mut Field
            return
        }
        acir(inline) fn f1 f1 {
          b0(v0: [[&mut Field; 1]; 1], v1: &mut &mut Field):
            v2 = array_get v0, index u32 0 -> [&mut Field; 1]
            v3 = array_get v2, index u32 0 -> &mut Field
            store v3 at v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let gets = collect_array_gets(&ssa); // [main's v3, main's v4]
        let loads = collect_loads(&ssa); // [main's v6]
        let mut analysis = analyze_main(&ssa);
        // v4 (deep extract before call) should alias v6 (loaded after call).
        assert!(analysis.may_alias(ssa.main(), gets[1], loads[0]));
    }

    /// Different reference types extracted
    /// from the same composite end up in the same alias class. They can never
    /// alias in reality.
    #[test]
    fn different_ref_types_in_struct_are_merged() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut [Field; 1]
            v2 = make_array [v0, v1] : [(&mut Field, &mut [Field; 1]); 1]
            v3 = array_get v2, index u32 0 -> &mut Field
            v4 = array_get v2, index u32 1 -> &mut [Field; 1]
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v3: &mut Field, v4: &mut [Field; 1] — cannot alias, but analysis put them in the same class.
        assert!(!analysis.may_alias(ssa.main(), gets[0], gets[1]));
    }

    /// `ValueId`s are scoped per-function in Noir SSA.
    /// The `GlobalValueId(FunctionId, ValueId)` key keeps each function's
    /// values in its own union-find namespace, so `f1`'s internal unification
    /// of its local `v0` and `v1` does not pollute `main`'s classes.
    #[test]
    fn value_ids_do_not_collide_across_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        acir(inline) fn f1 f1 {
          b0(v0: &mut Field):
            jmp b1(v0)
          b1(v1: &mut Field):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let main_allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        // Two independent allocations in `main` must not alias, even though
        // `f1` internally unifies its own v0 with its v1.
        assert!(!analysis.may_alias(ssa.main(), main_allocs[0], main_allocs[1]));
    }

    // ============================================================
    // Recursion
    // ============================================================

    /// Direct recursion: f1 calls itself with swapped arguments.
    /// The recursive call unifies f1's own formals with each other, which
    /// transitively unifies main's actuals through the shared formals.
    #[test]
    fn direct_recursion_unifies_through_swapped_args() {
        // v0 and v1 come from distinct oracles so they start in separate classes.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = call f1(v0, v1) -> &mut Field
            return
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field, v1: &mut Field):
            v2 = call f1(v1, v0) -> &mut Field
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        // call_results in main: [oracle_a's v0, oracle_b's v1, f1's v2].
        // f1's recursive call f1(v1, v0) swaps the formals via parameter
        // unification, so f1.formal_0 ~ f1.formal_1. Transitively, main's
        // v0 and v1 are unified through the shared formals.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    /// Mutual recursion with identity-style returns: f1 and f2 each call the
    /// other and return their own input. Under pure Steensgaard, argument and
    /// return-value unifications chain through both functions' formals and
    /// return slots, eventually linking main's argument to main's call result.
    #[test]
    fn mutual_recursion_unifies_through_shared_formals() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = call f1(v0) -> &mut Field
            return
        }
        brillig(inline) fn f1 f1 {
          b0(v0: &mut Field):
            v1 = call f2(v0) -> &mut Field
            return v0
        }
        brillig(inline) fn f2 f2 {
          b0(v0: &mut Field):
            v1 = call f1(v0) -> &mut Field
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let main = ssa.main();
        let call_result = {
            let mut result = None;
            for block in main.reachable_blocks() {
                for inst_id in main.dfg[block].instructions() {
                    if matches!(&main.dfg[*inst_id], Instruction::Call { .. }) {
                        result = Some(main.dfg.instruction_result::<1>(*inst_id)[0]);
                    }
                }
            }
            GlobalValueId::new(main, result.unwrap())
        };
        let mut analysis = analyze_main(&ssa);
        // main.v0 flows into f1.formal_0 (~ main.v0), f1 returns its formal,
        // so return_values[f1] ~ f1.formal_0 ~ main.v0 ~ main.call_result.
        assert!(analysis.may_alias(main, allocs[0], call_result));
    }

    // ============================================================
    // Return value handling
    // ============================================================

    /// Identity function returning its ref argument: the call result should
    /// alias the actual argument via the shared return slot.
    #[test]
    fn identity_function_aliases_argument_with_result() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = call f1(v0) -> &mut Field
            return
        }
        acir(inline) fn f1 f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let main = ssa.main();
        let call_result = {
            let mut result = None;
            for block in main.reachable_blocks() {
                for inst_id in main.dfg[block].instructions() {
                    if matches!(&main.dfg[*inst_id], Instruction::Call { .. }) {
                        result = Some(main.dfg.instruction_result::<1>(*inst_id)[0]);
                    }
                }
            }
            GlobalValueId::new(main, result.unwrap())
        };
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(main, allocs[0], call_result));
    }

    /// Function with two branches that flow to a shared return block through
    /// different block arguments. The shared return block's parameter unifies
    /// the two possible return values — transitively unifying the caller's
    /// actuals through the formal-to-return chain.
    #[test]
    fn multiple_return_paths_unify_through_shared_return_block() {
        // v1 and v2 come from two distinct foreign calls (oracles), so they
        // start in separate alias classes. Block b3's parameter in f1 joins the two
        // formals, transitively unifying main's v1 and v2.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = call oracle_a() -> &mut Field
            v2 = call oracle_b() -> &mut Field
            v3 = call f1(v0, v1, v2) -> &mut Field
            return
        }
        brillig(inline) fn f1 f1 {
          b0(v0: u1, v1: &mut Field, v2: &mut Field):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(v1)
          b2():
            jmp b3(v2)
          b3(v3: &mut Field):
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        // call_results in main: [oracle_a's v1, oracle_b's v2, f1's v3].
        // Block b3's parameter unifies f1.v1 (from b1) and f1.v2 (from b2)
        // via unify_with_block_params. Transitively, main's v1 ~ v2 through
        // the formals that were unified with b3's param.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    // ============================================================
    // Intrinsics (unify_on_signature)
    // ============================================================

    /// `vector_push_back` is handled precisely by `unify_vector_intrinsic`:
    /// the pushed element is linked into the *pointee* class of the
    /// vector (not merged with the container itself as a generic
    /// signature-level merge would do). This is the element-level
    /// semantics that matches what the intrinsic actually does.
    #[test]
    fn vector_push_back_links_pushed_element_into_pointee_class() {
        // v0 and v1 come from two distinct oracles.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field]
            v3, v4 = call vector_push_back(u32 1, v2, v1) -> (u32, [&mut Field])
            v5 = array_get v4, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // call_results: [oracle_a's v0, oracle_b's v1, push_back's v3 (u32), push_back's v4]
        let v0 = call_results[0];
        let v1 = call_results[1];
        let v5 = gets[0];
        // v5 (extracted from v4) aliases v0 (original element in v2).
        assert!(analysis.may_alias(ssa.main(), v0, v5));
        // Precise handling: v5 also aliases v1 (the pushed element),
        // because push_back links v1 into the new vector's pointee class.
        assert!(analysis.may_alias(ssa.main(), v1, v5));
        // Transitively, v0 and v1 alias through the shared pointee class.
        assert!(analysis.may_alias(ssa.main(), v0, v1));
    }

    /// `as_vector` converts an array to a vector: the two containers share
    /// elements, so refs extracted from the vector alias the original array's
    /// refs.
    #[test]
    fn as_vector_links_array_and_vector_elements() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            v2, v3 = call as_vector(v1) -> (u32, [&mut Field])
            v4 = array_get v3, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v4 (extracted from the vector) aliases v0 (original element of the array).
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[0]));
    }

    /// `vector_push_front` symmetric to `push_back`: pushed element lands in the
    /// new vector's pointee class.
    #[test]
    fn vector_push_front_links_pushed_element_into_pointee_class() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0] : [&mut Field]
            v3, v4 = call vector_push_front(u32 1, v2, v1) -> (u32, [&mut Field])
            v5 = array_get v4, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[0]));
        assert!(analysis.may_alias(ssa.main(), allocs[1], gets[0]));
    }

    /// `vector_pop_back`: the popped element was in the container's pointee
    /// class, so it aliases other elements of the same vector.
    #[test]
    fn vector_pop_back_links_popped_element_with_vector_contents() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field]
            v3, v4, v5 = call vector_pop_back(u32 2, v2) -> (u32, [&mut Field], &mut Field)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let main = ssa.main();
        // Locate the popped-element result (the last result of the call).
        let popped = {
            let mut result = None;
            for block in main.reachable_blocks() {
                for inst_id in main.dfg[block].instructions() {
                    if matches!(&main.dfg[*inst_id], Instruction::Call { .. }) {
                        let results = main.dfg.instruction_results(*inst_id);
                        result = Some(results[2]);
                    }
                }
            }
            GlobalValueId::new(main, result.unwrap())
        };
        let mut analysis = analyze_main(&ssa);
        // popped aliases v0 (originally in the vector's pointee class).
        assert!(analysis.may_alias(main, allocs[0], popped));
        // And v1 (also in the pointee class, field-insensitively).
        assert!(analysis.may_alias(main, allocs[1], popped));
    }

    /// `vector_pop_front`: layout differs (popped element comes before
    /// `new_len` and `new_vec` in the results list). Same aliasing effect.
    #[test]
    fn vector_pop_front_links_popped_element_with_vector_contents() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field]
            v3, v4, v5 = call vector_pop_front(u32 2, v2) -> (&mut Field, u32, [&mut Field])
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let main = ssa.main();
        // For pop_front, the popped element is the first result (results[0]).
        let popped = {
            let mut result = None;
            for block in main.reachable_blocks() {
                for inst_id in main.dfg[block].instructions() {
                    if matches!(&main.dfg[*inst_id], Instruction::Call { .. }) {
                        let results = main.dfg.instruction_results(*inst_id);
                        result = Some(results[0]);
                    }
                }
            }
            GlobalValueId::new(main, result.unwrap())
        };
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(main, allocs[0], popped));
        assert!(analysis.may_alias(main, allocs[1], popped));
    }

    /// `vector_insert`: inserted element lands in the pointee class, exactly
    /// like `push_back/push_front` but at an arbitrary index.
    #[test]
    fn vector_insert_links_inserted_element_into_pointee_class() {
        // Two oracles so v0 and v1 start in separate classes with ⊥ origin,
        // letting us observe the intrinsic-level aliasing without interference
        // from the allocation-site refinement.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field]
            v3, v4 = call vector_insert(u32 1, v2, u32 0, v1) -> (u32, [&mut Field])
            v5 = array_get v4, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // call_results: [oracle_a's v0, oracle_b's v1, insert's v3 (u32), insert's v4]
        let v0 = call_results[0];
        let v1 = call_results[1];
        let v5 = gets[0];
        // Inserted element (v1) aliases what was in the vector (v0).
        assert!(analysis.may_alias(ssa.main(), v0, v1));
        // And a later extract from the new vector aliases both.
        assert!(analysis.may_alias(ssa.main(), v1, v5));
    }

    /// `vector_remove`: removed element was in the vector's pointee class.
    #[test]
    fn vector_remove_links_removed_element_with_vector_contents() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v0, v1] : [&mut Field]
            v3, v4, v5 = call vector_remove(u32 2, v2, u32 0) -> (u32, [&mut Field], &mut Field)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let main = ssa.main();
        let removed = {
            let mut result = None;
            for block in main.reachable_blocks() {
                for inst_id in main.dfg[block].instructions() {
                    if matches!(&main.dfg[*inst_id], Instruction::Call { .. }) {
                        let results = main.dfg.instruction_results(*inst_id);
                        result = Some(results[2]);
                    }
                }
            }
            GlobalValueId::new(main, result.unwrap())
        };
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(main, allocs[0], removed));
        assert!(analysis.may_alias(main, allocs[1], removed));
    }

    // ============================================================
    // `unresolved_call` — conservative handling of opaque calls
    // (unresolved function pointers / indirect calls).
    // Tests exercise the four cases of the cascade:
    //   1. Same type         → merge_alias
    //   2. Containment       → merge_reference
    //   3. Shared inner ref  → merge_alias
    //   4. Complex relation  → escape
    //
    // Driven by an indirect `call v_fn(...)` through a `function`-typed
    // parameter — the genuinely-unresolved path. Foreign calls (e.g. `print`)
    // are NOT used here: references cannot be passed to foreign functions in
    // Noir, so the analysis handles them optimistically (no aliasing effect).
    // ============================================================

    /// Case 1: two args of the same ref type are merged via `merge_alias` —
    /// the callee might return one of them, or store one into the other's
    /// pointee.
    #[test]
    fn opaque_call_merges_same_typed_ref_args() {
        // Oracles so v0 and v1 start in separate classes.
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            call v10(v0, v1)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        // call_results: [oracle_a's v0, oracle_b's v1]. `print` is itself an
        // unresolved call that merges its two same-typed ref args.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
    }

    /// Case 2: a ref arg whose type is the element type of a composite arg is
    /// linked into the composite's pointee class via `merge_reference`. After
    /// the call, a ref extracted from the composite aliases the passed ref.
    #[test]
    fn opaque_call_links_ref_into_composite_pointee() {
        // `v4` is an unresolved (indirect) callee — the conservative path.
        let src = "
        brillig(inline) fn main f0 {
          b0(v4: function):
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v1] : [&mut Field; 1]
            call v4(v0, v2)
            v3 = array_get v2, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v0 joins pointee(v2) via merge_reference; v3 (extracted from v2)
        // also joins pointee(v2) → v0 and v3 are in the same class.
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[0]));
    }

    /// Case 3: two composite args that share an inner ref type but where
    /// neither type can contain the other. They're linked via `merge_alias`
    /// so their pointee classes share, and refs extracted from either alias
    /// with refs extracted from the other.
    #[test]
    fn opaque_call_merges_composites_sharing_inner_ref_type() {
        // Oracles so v0 and v1 start in separate classes —
        // the allocation-site refinement stays out of the way.
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field; 1]
            v3 = make_array [v1, v1] : [&mut Field; 2]
            call v10(v2, v3)
            v4 = array_get v2, index u32 0 -> &mut Field
            v5 = array_get v3, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // [&mut Field; 1] and [&mut Field; 2] are distinct Type values
        // (different SemanticLength), so neither contains the other. They
        // share &mut Field → Case 3 fires, merging their containers.
        // Their pointee classes share, so v4 (from v2) aliases v5 (from v3),
        // and transitively v0 aliases v1.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
        assert!(analysis.may_alias(ssa.main(), gets[0], gets[1]));
    }

    /// Two args of completely unrelated ref types — no merge. The
    /// analysis leaves unrelated values in distinct classes even when passed
    /// through an opaque call.
    #[test]
    fn opaque_call_does_not_merge_unrelated_ref_types() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut u32
            call v10(v0, v2)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v0 and v2 have unrelated ref types (&Field vs &u32): no merge.
        // v1 is another &Field that wasn't involved in any call.
        // So v0 and v1 stay in separate classes — no spurious aliasing.
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
    }

    /// Case 3 : when two args have types that neither
    /// contain each other nor share a common ref type, but one's pointee
    /// appears as a structural field of the other's pointee, the callee
    /// could create a sub-reference.
    #[test]
    fn opaque_call_with_sub_ref_pattern_marks_values_as_escaped() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut [Field; 2]
            v2 = allocate -> &mut Field
            v3 = allocate -> &mut u32
            call v10(v0, v1)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        // v0 and v2 are &Field but v2 is not involved in the call and
        // cannot alias with v1
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[2]));
        // v3 is &u32, a different type: the type check rejects it
        // preserving at least some precision.
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[3]));
        // Escape marks v0 and v1 as aliased — even though their classes are
        // singletons.
        assert!(analysis.is_aliased(allocs[0]));
        assert!(analysis.is_aliased(allocs[1]));
        // v2 wasn't in any call → not aliased.
        assert!(!analysis.is_aliased(allocs[2]));
    }

    /// `type_representatives` collapses N entries of the same type into a single
    /// alias class via one `merge_alias` per additional entry — no pairwise loop.
    /// This test exercises N=4 to confirm the bucket merge reaches every entry,
    /// not just the first pair.
    #[test]
    fn unresolved_call_buckets_n_same_typed_refs_into_one_class() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = call oracle_c() -> &mut Field
            v3 = call oracle_d() -> &mut Field
            call v10(v0, v1, v2, v3)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let mut analysis = analyze_main(&ssa);
        // All four refs share one bucket → one alias class.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[2]));
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[3]));
        assert!(analysis.may_alias(ssa.main(), call_results[1], call_results[2]));
        assert!(analysis.may_alias(ssa.main(), call_results[2], call_results[3]));
    }

    /// One unresolved call with two same-typed inner refs and one outer composite
    /// that can contain them. The bucket merge collapses the two inner refs;
    /// the cross-bucket containment links the (single) bucket rep into the
    /// outer's pointee. After extracting from the composite, the result aliases
    /// both inner refs through the merged pointee class.
    #[test]
    fn unresolved_call_combines_bucket_merge_and_cross_bucket_containment() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_inner_a() -> &mut Field
            v1 = call oracle_inner_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field; 1]
            call v10(v0, v1, v2)
            v3 = array_get v2, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // Bucket: {v0, v1} both `&mut Field`, merged via `type_representatives`.
        assert!(analysis.may_alias(ssa.main(), call_results[0], call_results[1]));
        // Cross-bucket Case 2: bucket rep linked into v2's pointee. `array_get v2`
        // extracts a value that's also v2's pointee, so v3 aliases v0 and v1.
        assert!(analysis.may_alias(ssa.main(), call_results[0], gets[0]));
        assert!(analysis.may_alias(ssa.main(), call_results[1], gets[0]));
    }

    /// The signature cache is keyed on the *sorted* `Vec<Type>`, so two opaque
    /// calls with the same set of arg types in different orderings hit the same
    /// cache slot. Behaviorally the resulting alias merges must be identical.
    #[test]
    fn unresolved_call_signature_is_argument_order_invariant() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v10: function):
            v0 = call oracle_inner_a() -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            call v10(v0, v1)
            v2 = call oracle_inner_b() -> &mut Field
            v3 = make_array [v2] : [&mut Field; 1]
            call v10(v3, v2)
            v4 = array_get v1, index u32 0 -> &mut Field
            v5 = array_get v3, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let gets = collect_array_gets(&ssa);
        let mut analysis = analyze_main(&ssa);
        // First call had args in order (ref, array); second in (array, ref).
        // Both must produce the same Case-2 merge: ref becomes pointee of array.
        assert!(analysis.may_alias(ssa.main(), call_results[0], gets[0]));
        assert!(analysis.may_alias(ssa.main(), call_results[1], gets[1]));
    }

    /// Globals are compile-time constants in Noir and cannot carry references
    /// (per `GlobalsGraph`'s invariant). They flow through the analysis
    /// transparently: the `contains_reference()` filter rejects them
    /// everywhere they appear, so they never enter the union-find nor affect
    /// any alias class.
    ///
    /// This test verifies that a function that uses a global alongside
    /// ordinary reference-carrying values still produces correct alias
    /// information — the global doesn't spuriously link unrelated refs.
    #[test]
    fn globals_do_not_interfere_with_alias_analysis() {
        let src = "
        g0 = make_array [Field 1, Field 2, Field 3] : [Field; 3]

        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = array_get g0, index u32 0 -> Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        // Two distinct allocates of &mut Field should not alias, even with
        // a global array access interleaved.
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
    }

    /// `array_set` on a global produces a new array (SSA is immutable). The
    /// global itself is unchanged; the new array is independent. Since the
    /// global's type carries no references, the `merge_reference` calls in
    /// the `ArraySet` handler are no-ops, and neither the global nor the
    /// result enters the union-find.
    ///
    /// This test confirms the analysis handles `array_set` on a global
    /// without panicking and without leaking any alias relationships to
    /// unrelated values.
    #[test]
    fn array_set_on_global_does_not_interfere() {
        let src = "
        g0 = make_array [Field 1, Field 2, Field 3] : [Field; 3]

        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = array_set g0, index u32 0, value Field 99
            v3 = array_get v2, index u32 0 -> Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let mut analysis = analyze_main(&ssa);
        // Unrelated allocates stay in distinct classes — the global mutation
        // does not pollute the analysis.
        assert!(!analysis.may_alias(ssa.main(), allocs[0], allocs[1]));
    }

    /// When a composite is put inside another composite,
    /// the inner composite is linked into the outer's pointee class,
    /// so refs reachable through multiple levels of nesting must alias.
    #[test]
    fn nested_composites_propagate_inner_refs() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            v2 = make_array [v1] : [[&mut Field; 1]; 1]
            v3 = array_get v2, index u32 0 -> [&mut Field; 1]
            v4 = array_get v3, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa); // [v0]
        let gets = collect_array_gets(&ssa); // [v3 (composite), v4 (ref)]
        let mut analysis = analyze_main(&ssa);
        // ref_a (v0) and ref_out (v4) should alias under field-insensitivity:
        // v4 is extracted from v3 which is extracted from v2 which holds v1 which holds v0.
        assert!(analysis.may_alias(ssa.main(), allocs[0], gets[1]));
    }

    // ============================================================
    // must_alias
    // ============================================================

    /// Result `ValueIds` of every `IfElse` in main, in declaration order.
    fn collect_ifelse_results(ssa: &Ssa) -> Vec<GlobalValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::IfElse { .. }) {
                    let id =
                        GlobalValueId::new(func, func.dfg.instruction_result::<1>(*inst_id)[0]);
                    out.push(id);
                }
            }
        }
        out
    }

    #[test]
    fn must_alias_same_value() {
        // The `a == b` short-circuit fires regardless of whether the value
        // has a tracked allocation site.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let calls = collect_call_results_in_main(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(calls[0], calls[0]));
    }

    #[test]
    fn must_alias_two_distinct_allocates_false() {
        // Two `Allocate` instructions yield distinct sites; must_alias is false
        // even though both are tracked.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(!analysis.must_alias(allocs[0], allocs[1]));
    }

    #[test]
    fn must_alias_via_block_param_join() {
        // The block parameter of b1 inherits v0's allocation site (single-pred
        // join in track_allocations_from_predecessors). v0 and the block param
        // are distinct SSA values but share a site → must-alias.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1(v0)
          b1(v1: &mut Field):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let blocks: Vec<_> = ssa.main().reachable_blocks().into_iter().collect();
        let b1_param = GlobalValueId::new(ssa.main(), ssa.main().dfg[blocks[1]].parameters()[0]);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(allocs[0], b1_param));
    }

    #[test]
    fn must_alias_via_ifelse_same_site() {
        // v1 (allocate) has site Some(v1); v2 (block-param) inherits Some(v1).
        // IfElse joining v1 and v2 produces a result with site Some(v1) via
        // AllocationLattice::join. v1 and the IfElse result are distinct SSA
        // values that must-alias.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            jmp b1(v1)
          b1(v2: &mut Field):
            v3 = not v0
            v4 = if v0 then v1 else (if v3) v2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let ifelse_results = collect_ifelse_results(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(allocs[0], ifelse_results[0]));
    }

    #[test]
    fn must_alias_ifelse_mixed_sites_false() {
        // IfElse joining two distinct allocations: the lattice join collapses
        // to NoAllocation, so the result has no site → must_alias is false.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            v3 = not v0
            v4 = if v0 then v1 else (if v3) v2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let ifelse_results = collect_ifelse_results(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(!analysis.must_alias(allocs[0], ifelse_results[0]));
        assert!(!analysis.must_alias(allocs[1], ifelse_results[0]));
    }

    #[test]
    fn must_alias_via_load_single_store() {
        // Pass 2 propagates the pointee_sites lattice into Load results.
        // When only `v1` has been stored at `*v0`, `pointee_sites[*v0]` is
        // `Known(v1)`, so the loaded value inherits site `Some(v1)` and
        // must-aliases v1.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut &mut Field
            v1 = allocate -> &mut Field
            store v1 at v0
            v2 = load v0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(allocs[1], loads[0]));
    }

    #[test]
    fn must_alias_via_load_mixed_stores_false() {
        // Two distinct allocations have been stored at `*v0`, so the lattice
        // collapses to `NoAllocation`. Pass 2 sets no site on the loaded value
        // and must_alias returns false.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut &mut Field
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut Field
            store v1 at v0
            store v2 at v0
            v3 = load v0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(!analysis.must_alias(allocs[1], loads[0]));
        assert!(!analysis.must_alias(allocs[2], loads[0]));
    }

    #[test]
    fn must_alias_via_array_get_single_make_array() {
        // `MakeArray` joins each element's site into the array's pointee
        // class. When every element is `v0`, the lattice is `Known(v0)` and
        // `array_get` inherits site `Some(v0)`.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = make_array [v0, v0] : [&mut Field; 2]
            v2 = array_get v1, index u32 0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let gets = collect_array_gets(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(allocs[0], gets[0]));
    }

    #[test]
    fn must_alias_ifelse_over_load_result() {
        // Pass 2b: an IfElse over a load result picks up the load's site
        // (set by pass 2a). Both branches resolve to `Some(v1)`, so the
        // IfElse result must-aliases v1.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut &mut Field
            store v1 at v2
            v3 = load v2 -> &mut Field
            v4 = not v0
            v5 = if v0 then v1 else (if v4) v3
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let ifelse_results = collect_ifelse_results(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(analysis.must_alias(allocs[0], ifelse_results[0]));
    }

    /// loop-Allocate
    ///
    /// `v_local = allocate` lives inside a CFG loop, so it fires once per
    /// iteration and produces a fresh cell each time. `v_load = load v_p`
    /// reads `*v_p` *before* this iteration's store, so on iteration N>1
    /// it sees iteration N-1's cell while `v_local` refers to iteration
    /// N's cell. Different runtime cells, both with static site
    /// `Known(v_local)` (same `Allocate` instruction).
    ///
    /// `must_alias(v_local, v_load)` must return false.
    #[test]
    fn must_alias_sound_on_loop_local_allocate() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut &mut Field
            jmp b1(v0)
          b1(v2: u1):
            v3 = load v1 -> &mut Field
            v4 = allocate -> &mut Field
            store v4 at v1
            jmpif v2 then: b1(v2), else: b2()
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        // allocs[0] = v1 (the heap-style pointer, allocated outside the loop)
        // allocs[1] = v4 (the in-loop allocate — fresh cell every iteration)
        // loads[0]  = v3 (loads `*v1` before this iteration's store)
        //
        // Sound answer: false. v3 reads a previous iteration's cell of v4;
        // the in-loop v4 refers to this iteration's fresh cell.
        assert!(
            !analysis.must_alias(allocs[1], loads[0]),
            "must_alias unsoundly returns true for two values that may be \
             from different loop iterations of the same Allocate instruction"
        );
    }

    #[test]
    fn foreign_call_preserves_local_sites_under_no_escape() {
        // A foreign call cannot reenter program code, so it does not flag
        // the calling function as recursive. Combined with the no-escape
        // invariant — function-local allocations cannot leak into the
        // caller's pointee chains — sites stored before the call survive,
        // and post-call loads through the chain still must-alias the
        // originally stored values.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            v2 = allocate -> &mut &mut &mut Field
            store v1 at v2
            call oracle_op(v2)
            v3 = load v2 -> &mut &mut Field
            v4 = load v3 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        // loads[0] = v3 (load v2), loads[1] = v4 (load v3)
        assert!(analysis.must_alias(allocs[1], loads[0]));
        assert!(analysis.must_alias(allocs[0], loads[1]));
    }

    #[test]
    fn entry_point_parameter_pointees_are_poisoned() {
        // For entry-point parameters the "caller" is external and may
        // have stashed any value into the parameters' pointee chains
        // before the function ran. Both `*v0` and any deeper level
        // reachable from a parameter must be treated as `NoAllocation`,
        // so an in-function store does not become observable as the
        // load result's site.
        //
        // v0 is a parameter with a deep ref chain. We store a known-site
        // ref into *v0; without poisoning, the load would inherit that
        // site — wrong, because the external caller could have placed
        // a different reference there before main started.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut &mut Field):
            v1 = allocate -> &mut Field
            store v1 at v0
            v2 = load v0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        // The store happened *after* the entry-point poison, so the
        // class is NoAllocation; the load's result has no site.
        assert!(!analysis.must_alias(allocs[0], loads[0]));
    }

    #[test]
    fn may_alias_local_vs_entry_param_pointee_false() {
        // Local Allocate cannot alias content of an external pointee chain.
        let src = "
    acir(inline) fn main f0 {
      b0(v0: &mut &mut Field):
        v1 = allocate -> &mut Field
        v2 = load v0 -> &mut Field   // v2.site = External after pass 2
        return
    }
    ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        assert!(!analysis.must_alias(allocs[0], loads[0]));
    }

    /// Multi-call-site site propagation
    ///
    /// Steensgaard merges all call sites of `f1` into a single alias
    /// class, so `points_to_sites` for `f1::outer`'s pointee class —
    /// set to `Known(f1::inner)` by the store inside `f1` — becomes
    /// the site of every load through any call result. Pass 2 writes
    /// `Known(f1::inner)` into both `main.v1` (load through the first
    /// call's result) and `main.v3` (load through the second call's
    /// result). The two `inner` cells are distinct at runtime — each
    /// call to `f1` allocates a fresh one — so `must_alias` between
    /// them must be `false`.
    #[test]
    fn must_alias_sound_on_multi_call_site_non_recursive_callee() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call f1() -> &mut &mut Field
            v1 = load v0 -> &mut Field
            v2 = call f1() -> &mut &mut Field
            v3 = load v2 -> &mut Field
            return
        }
        brillig(inline) fn f1 f1 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        // loads[0] = main.v1 (first call's `inner` cell)
        // loads[1] = main.v3 (second call's `inner` cell — distinct)
        assert!(
            !analysis.must_alias(loads[0], loads[1]),
            "must_alias unsoundly returns true for values from two \
             distinct call sites of a non-recursive callee — each call \
             to f1 allocates a fresh `inner` cell"
        );
    }

    /// Multi-call-site site propagation under a caller-side loop
    ///
    /// A variant of the multi-call-site case in which the callee is
    /// invoked from inside a loop in the caller. The callee's
    /// allocation lives outside any loop in *its own* function, so
    /// `loop_blocks` does not flag it as `loop_allocates`; the callee
    /// is also non-recursive. Yet each loop iteration in the caller
    /// allocates a fresh `f1::inner`, so `must_alias` between two
    /// load results from different iterations should be `false`.
    #[test]
    fn must_alias_sound_on_callee_alloc_with_loop_callsite() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1(v0)
          b1(v1: u1):
            v2 = call f1() -> &mut &mut Field
            v3 = load v2 -> &mut Field
            jmpif v1 then: b1(v1), else: b2()
          b2():
            return
        }
        brillig(inline) fn f1 f1 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let loads = collect_loads(&ssa);
        let analysis = analyze_main(&ssa);
        // Only one Load instruction (in b1), but it executes once per
        // iteration. Two iterations see two distinct `f1::inner` cells.
        // Because the analysis associates a *single* GlobalValueId with
        // the load, we cannot assert across iterations directly here —
        // but the trusted site `Known(f1::inner)` is enough to fool any
        // consumer that compares loads across iterations via must_alias.
        // The companion load_store_forwarding test exercises that path.
        // We assert here that the load's site is *not* a trusted one —
        // i.e. that nothing trusts `Known(f1::inner)` for this load.
        assert!(
            analysis.known_site(loads[0]).is_none(),
            "the load's allocation site is trusted, but each loop \
             iteration produces a fresh `f1::inner` cell — any consumer \
             treating this site as a must-alias key is unsound"
        );
    }

    // ============================================================
    // Single-function scope (`analyze_single_function`)
    //
    // The whole SSA is unavailable: every call is opaque, and the analyzed
    // function is its own entry point. Site trust follows the "untrust any
    // caller" rule — a function that calls any other Noir function has all
    // its local allocations classified `Multiple`.
    // ============================================================

    /// A call-free function: its single allocate is a trusted (`Known`) site.
    #[test]
    fn single_fn_call_free_allocate_is_known() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert_eq!(analysis.known_site(allocs[0]), Some(allocs[0]));
    }

    /// Two distinct allocates in a call-free function have distinct trusted
    /// sites: they do not must-alias and provably cannot be equal.
    #[test]
    fn single_fn_distinct_allocates_cannot_equal() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert!(!analysis.must_alias(allocs[0], allocs[1]));
        assert!(analysis.cannot_equal(allocs[0], allocs[1]));
        assert_eq!(analysis.known_site(allocs[0]), Some(allocs[0]));
        assert_eq!(analysis.known_site(allocs[1]), Some(allocs[1]));
    }

    /// In a call-free function a load through a single-stored pointer recovers
    /// the stored value's site, so the two must-alias.
    #[test]
    fn single_fn_load_after_store_recovers_must_alias() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut &mut Field
            store v0 at v1
            v2 = load v1 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert!(analysis.must_alias(allocs[0], loads[0]));
        assert_eq!(analysis.known_site(loads[0]), Some(allocs[0]));
    }

    /// A loop-resident allocate fires once per iteration, so even a call-free
    /// function classifies it `Multiple` — no trusted site.
    #[test]
    fn single_fn_loop_allocate_is_multiple() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1(v0)
          b1(v1: u1):
            v2 = allocate -> &mut Field
            jmpif v1 then: b1(v1), else: b2()
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert_eq!(analysis.known_site(allocs[0]), None);
    }

    /// "Untrust any caller": the same allocate is `Known` with no call present
    /// but `Multiple` (untrusted) once the function calls another function,
    /// because the opaque callee could transitively re-enter this function.
    #[test]
    fn single_fn_any_call_untrusts_local_allocates() {
        let call_free = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(call_free).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert_eq!(analysis.known_site(allocs[0]), Some(allocs[0]));

        let with_call = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            call f1()
            return
        }
        acir(inline) fn f1 f1 {
          b0():
            return
        }
        ";
        let ssa = Ssa::from_str(with_call).unwrap();
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert_eq!(analysis.known_site(allocs[0]), None);
    }

    /// An out-of-scope call's result is opaque: it carries no trusted site.
    #[test]
    fn single_fn_opaque_call_result_has_no_site() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = call f1() -> &mut Field
            return
        }
        acir(inline) fn f1 f1 {
          b0():
            v0 = allocate -> &mut Field
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let call_results = collect_call_results_in_main(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert_eq!(analysis.known_site(call_results[0]), None);
    }

    /// An entry-point reference parameter is `External`: it cannot be equal to
    /// a cell allocated locally after entry.
    #[test]
    fn single_fn_entry_param_cannot_equal_local_allocate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field):
            v1 = allocate -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();
        let param = GlobalValueId::new(main, main.dfg[main.entry_block()].parameters()[0]);
        let allocs = collect_allocates(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(main);
        assert!(analysis.cannot_equal(param, allocs[0]));
    }

    /// An entry-point parameter's pointees are poisoned: a value stored through
    /// the parameter and then loaded back does not must-alias the stored value,
    /// because the external caller's pre-entry writes are opaque.
    #[test]
    fn single_fn_entry_param_pointee_poisoned() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut &mut Field):
            v1 = allocate -> &mut Field
            store v1 at v0
            v2 = load v0 -> &mut Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let allocs = collect_allocates(&ssa);
        let loads = collect_loads(&ssa);
        let analysis = AliasAnalysis::analyze_single_function(ssa.main());
        assert!(!analysis.must_alias(allocs[0], loads[0]));
    }

    #[test]
    fn allocation_site_propagates_to_loop_header_param_via_fixed_point() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            jmp b1(v0)
          b1(v1: &mut Field):
            jmpif u1 0 then: b2(v1), else: b3()
          b2(v2: &mut Field):
            jmp b1(v2)
          b3():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let func = ssa.main();
        let allocs = collect_allocates(&ssa);
        let loop_header = func.reachable_blocks().into_iter().find(|block| {
            let params = func.dfg[*block].parameters();
            params.len() == 1 && *block != func.entry_block()
        });
        let loop_param = func.dfg[loop_header.unwrap()].parameters()[0];

        let mut analysis = analyze_main(&ssa);
        let loop_param = GlobalValueId::new(func, loop_param);
        assert!(analysis.may_alias(func, allocs[0], loop_param));
        assert!(analysis.must_alias(allocs[0], loop_param));
    }
}
