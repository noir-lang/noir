// The alias analysis is not consumed by any optimization pass yet,
// remove it once the analysis is used.
#![allow(dead_code)]

//! Steensgaard-style alias analysis for SSA references.
//!
//! This is a flow-insensitive, unification-based alias analysis. It walks every
//! instruction in no specific order and builds equivalence classes (alias sets) of
//! references that may point to the same memory location.
//!
//! On top of Steensgaard analysis, we also collect allocation sites and propagate their definition when possible.
//! In order to benefit the most from the allocation tracking, we process the blocks in RPO,
//! to ensure allocation site are collected in the predecessors of a block before being
//! propagated in the block. Any other order will still be sound but much less precise.
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
//! 1. unify shared 'globals having reference' across functions in the GlobalValueId type
//! 2. add them to the signature-based analysis.
//!
//! ## Algorithm outline
//!
//! Each reference-typed value is assigned into a single alias set via a union-find structure.
//! Instructions update the alias sets using the 4 constraints: a = b, a = &b, a = *b, *a = b
//! Allocate instructions are tracked in the `allocation_sites` map, and propagate among blocks
//! following the terminator arguments (if all predecessor arguments have the same allocation site).
//!
//! The analysis is inter-procedural: arguments and parameters are unified for all call sites of a function
//! as well as the results and the function's return values.
//! To make the analysis order independent, the unified returned values of a function is stored (and updated after every call)
//! and initialized either by the results or the returned values (depending and which one comes first)
//! Because ValueIds are per function, we have to reason instead on GlobalValueId (FunctionId, ValueId).
//!
//! Unresolved function calls are not handled in Steensgaard analysis.
//! We do a conservative analysis based on the types of the function's signature.
//! This part is quadratic in the number of arguments/results, but run once per canonicalized signature.
//! The classic Steensgaard analysis is quasi-linear.
//!
//! Supporting unresolved function calls allows us to do the analysis for a single function.
//! Thus the analysis can be run on the whole program (recommended) or for one function in isolation.
//!
//! After processing all instructions, the union-find partitions every reference
//! into alias classes. Two references are *may-alias* if and only if they
//! belong to the same class, have the same type and have no distinct known allocation site.
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
        cfg::ControlFlowGraph,
        function::{Function, FunctionId},
        instruction::{Instruction, Intrinsic, TerminatorInstruction},
        post_order::PostOrder,
        types::{CompositeType, Type},
        union_find::UnionFind,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

/// Canonicalize a type so that `&T` and `&mut T` are treated as the same type
/// for aliasing. Every `Reference(_, _)` is rewritten to have `mutable = false`.
fn canonicalize_type(typ: &Type) -> Type {
    match typ {
        Type::Reference(inner, _) => Type::Reference(Arc::new(canonicalize_type(inner)), false),
        Type::Array(composite, size) => {
            let slots: CompositeType = composite.iter().map(canonicalize_type).collect();
            Type::Array(Arc::new(slots), *size)
        }
        Type::Vector(composite) => {
            let slots: CompositeType = composite.iter().map(canonicalize_type).collect();
            Type::Vector(Arc::new(slots))
        }
        Type::Numeric(_) | Type::Function => typ.clone(),
    }
}

/// Scope of the analysis
enum Scope<'a> {
    /// Analyze a single function
    Single(&'a Function),
    /// Analyze the whole program. More precise; preferred over the single-function analysis.
    Ssa(&'a Ssa),
}

impl Scope<'_> {
    fn is_entry_point(&self, function_id: FunctionId) -> bool {
        match self {
            Scope::Single(function) => function.id() == function_id,
            Scope::Ssa(ssa) => ssa.is_entry_point(function_id),
        }
    }

    fn functions(&self) -> Vec<&Function> {
        match self {
            Scope::Single(f) => vec![*f],
            Scope::Ssa(ssa) => ssa.functions.values().collect(),
        }
    }
}

/// GlobalValueId are ValueId along with their FunctionId,
/// allowing to globally use ValueIds coming from several functions.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct GlobalValueId(FunctionId, ValueId);

impl GlobalValueId {
    fn new(function: &Function, value: ValueId) -> Self {
        GlobalValueId(function.id(), value)
    }
}

pub(crate) struct AliasAnalysis {
    /// union-find structure mapping GlobalValueId to their alias class.
    aliases: UnionFind<GlobalValueId>,

    /// Maps an alias class representative to the alias class of what it points to.
    points_to: HashMap<GlobalValueId, GlobalValueId>,

    /// Number of values in each alias class, keyed by the class representative.
    /// Populated lazily on the first `is_aliased` call so that `analyze` stays
    /// cheap for consumers that only query `may_alias`.
    class_sizes: Option<HashMap<GlobalValueId, u32>>,

    /// Track known allocation sites: map a value to the `Allocate` that defined it.
    /// This is used to recover precision by saying that two values having
    /// two distinct allocation sites cannot alias.
    allocation_sites: HashMap<GlobalValueId, GlobalValueId>,
}

impl AliasAnalysis {
    /// Run alias analysis on all `functions` and return the computed alias sets.
    ///
    /// The constraints are monotone and converge in a single pass.
    pub(crate) fn analyze(ssa: &Ssa) -> Self {
        AliasAnalysisContext::analyze_with_scope(Scope::Ssa(ssa))
    }
    /// Build an analysis for one function in isolation. All calls are treated as
    /// opaque via `unresolved_call`. Less precise than [`Self::analyze`] but usable
    /// when the whole SSA is unavailable.
    pub(crate) fn analyze_single_function(function: &Function) -> Self {
        AliasAnalysisContext::analyze_with_scope(Scope::Single(function))
    }

    /// Returns `true` if `a` and `b` may refer to the same memory location.
    ///
    /// Takes `&mut self` because of path compression.
    /// This has no visible side-effect and is perfectly safe.
    pub(crate) fn may_alias(&mut self, function: &Function, a: ValueId, b: ValueId) -> bool {
        if a == b {
            return true;
        }

        // Field-insensitivity may alias values with distinct types, but such values cannot alias.
        // Types are canonicalized first so `&T` and `&mut T` compare equal
        let type_a = canonicalize_type(&function.dfg.type_of_value(a));
        let type_b = canonicalize_type(&function.dfg.type_of_value(b));
        if type_a != type_b {
            return false;
        }
        let a = GlobalValueId::new(function, a);
        let b = GlobalValueId::new(function, b);
        if let Some(allocate_a) = self.allocation_sites.get(&a)
            && let Some(allocate_b) = self.allocation_sites.get(&b)
            && allocate_a != allocate_b
        {
            return false;
        }

        let a_root = self.aliases.find(a);
        let b_root = self.aliases.find(b);

        a_root == b_root
    }

    /// Returns `true` if `value` may be aliased with some other value in the program.
    ///
    /// The per-class size table is populated on demand the first time this is
    /// called — consumers that only use [`Self::may_alias`] do not pay for it.
    pub(crate) fn is_aliased(&mut self, function: &Function, value: ValueId) -> bool {
        let rep = GlobalValueId::new(function, value);
        let root = self.aliases.find(rep);
        if self.class_sizes.is_none() {
            // Count members per alias class
            self.class_sizes = Some(self.aliases.class_sizes());
        }
        let sizes = self.class_sizes.as_ref().expect("just populated");
        !matches!(sizes.get(&root), None | Some(1))
    }

    /// Recursively check if `target` can be referenced by `from`
    pub(crate) fn may_reference(
        &mut self,
        function: &Function,
        from: ValueId,
        target: ValueId,
    ) -> bool {
        let mut seen = HashSet::default();
        let from = self.aliases.find(GlobalValueId::new(function, from));
        let target = self.aliases.find(GlobalValueId::new(function, target));
        if from == target {
            if let Some(allocate_a) = self.allocation_sites.get(&from)
                && let Some(allocate_b) = self.allocation_sites.get(&target)
                && allocate_a != allocate_b
            {
                return false;
            }
            return true;
        }
        let mut current = from;
        while seen.insert(current) {
            match self.points_to.get(&current) {
                Some(&next) => {
                    let next = self.aliases.find(next);
                    if next == target {
                        return true;
                    }
                    current = next;
                }
                None => return false,
            }
        }
        false
    }
}

#[derive(Clone, Copy)]
enum AllocationLattice {
    Undef,
    Known(GlobalValueId),
    NoAllocation,
}

impl AllocationLattice {
    fn join(self, other: Self) -> Self {
        use AllocationLattice::*;
        match (self, other) {
            (Undef, x) | (x, Undef) => x,
            (NoAllocation, _) | (_, NoAllocation) => NoAllocation,
            (Known(a), Known(b)) if a == b => Known(a),
            _ => NoAllocation,
        }
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

/// AliasAnalysis stores the result of the alias analysis pass
/// as well as transient data computed during the analysis
#[derive(Default)]
struct AliasAnalysisContext {
    /// union-find structure mapping GlobalValueId to their alias class.
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

    /// Known allocation sites
    allocation_sites: HashMap<GlobalValueId, GlobalValueId>,

    /// Signature Templates: cache template rules for a canonicalized signature
    /// represented by a vector of (distinct and ordered) Types
    signatures: HashMap<Vec<Type>, Vec<SignatureTemplate>>,
}

impl AliasAnalysisContext {
    fn analyze_with_scope(scope: Scope) -> AliasAnalysis {
        // Precondition: globals are expected to be pure constants (numeric or
        // composite-of-numeric) as documented.
        let functions = scope.functions();
        if let Some(first) = functions.first() {
            for (_, global) in first.dfg.globals.values_iter() {
                assert!(
                    !global.get_type().contains_reference(),
                    "ICE: alias_analysis assumes globals do not have references"
                );
            }
        }

        let mut analysis = Self::default();

        for function in functions {
            analysis.analyze_function(&scope, function);
        }

        AliasAnalysis {
            aliases: analysis.aliases,
            points_to: analysis.points_to,
            class_sizes: None,
            allocation_sites: analysis.allocation_sites,
        }
    }

    /// Walk every block in one function, processing instructions and terminators.
    /// If the function is an entry point of the SSA, also unify its
    /// same-typed reference parameters.
    fn analyze_function(&mut self, scope: &Scope, function: &Function) {
        if scope.is_entry_point(function.id()) {
            // Unify the reference parameters of the entry point because the
            // external caller may pass the same reference to 2 reference parameters.
            let params = function.dfg[function.entry_block()].parameters();
            self.unresolved_call(function, params, &[]);
        }

        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_cfg(&cfg);
        let blocks = post_order.into_vec_reverse();

        for block_id in blocks {
            self.track_allocations_from_predecessors(block_id, function, &cfg);
            self.analyze_block(function, block_id, scope);
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

    fn get_allocation(&self, arg: GlobalValueId) -> AllocationLattice {
        match self.allocation_sites.get(&arg) {
            Some(site) => AllocationLattice::Known(*site),
            None => AllocationLattice::NoAllocation,
        }
    }

    fn track_allocations_from_predecessors(
        &mut self,
        block_id: BasicBlockId,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) {
        let params = function.dfg[block_id].parameters();
        let mut allocations = vec![AllocationLattice::Undef; params.len()];
        let mut meet_arguments = |args: &[ValueId]| {
            for (i, &arg) in args.iter().enumerate() {
                let l = self.get_allocation(GlobalValueId::new(function, arg));
                allocations[i] = allocations[i].join(l);
            }
        };
        for predecessor in cfg.predecessors(block_id) {
            match function.dfg[predecessor].terminator().unwrap() {
                TerminatorInstruction::JmpIf {
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    ..
                } => {
                    if *then_destination == block_id {
                        meet_arguments(then_arguments);
                    }
                    if *else_destination == block_id {
                        meet_arguments(else_arguments);
                    }
                }
                TerminatorInstruction::Jmp { destination, arguments, .. } => {
                    debug_assert_eq!(*destination, block_id);
                    meet_arguments(arguments);
                }
                TerminatorInstruction::Return { .. }
                | TerminatorInstruction::Unreachable { .. } => {
                    unreachable!("ICE - unreachable predecessor block")
                }
            }
        }

        for (&param, allocation) in params.iter().zip(allocations) {
            if let AllocationLattice::Known(site) = allocation {
                self.allocation_sites.insert(GlobalValueId::new(function, param), site);
            }
        }
    }

    /// Process all instructions in a single block, updating the alias sets.
    fn analyze_block(&mut self, function: &Function, block_id: BasicBlockId, scope: &Scope) {
        let block = &function.dfg[block_id];

        for instruction_id in block.instructions() {
            match &function.dfg[*instruction_id] {
                Instruction::Allocate => {
                    // Defines a new pointer value.
                    let address = function.dfg.instruction_result::<1>(*instruction_id)[0];
                    let address = GlobalValueId::new(function, address);
                    self.aliases.make_set(address);
                    self.allocation_sites.insert(address, address);
                }
                Instruction::Load { address } => {
                    // Complex constraint type 1: result = *address
                    let result = function.dfg.instruction_result::<1>(*instruction_id)[0];
                    self.merge_reference(function, result, *address);
                }
                Instruction::Store { address, value } => {
                    // Complex constraint type 2: *address = value
                    self.merge_reference(function, *value, *address);
                }
                Instruction::Call { func: callee_id, arguments } => {
                    let results = function.dfg.instruction_results(*instruction_id);
                    match (&function.dfg[*callee_id], scope) {
                        // Inter-procedural analysis for resolved functions
                        // - merge arguments with their parameters,
                        // - process the function body (i.e analyze the instructions, but only once since it context-insensitive).
                        //   This is done through analyze_function() which process all the functions.
                        // - merge return values with the instruction results
                        (Value::Function(callee_id), Scope::Ssa(ssa)) => {
                            self.unify_call_arguments_and_return(
                                ssa, *callee_id, function, arguments, results,
                            );
                        }
                        (Value::Intrinsic(Intrinsic::Hint(_)), _) => {
                            self.unresolved_call(function, arguments, results);
                        }
                        (Value::Intrinsic(intrinsic), _)
                            if Self::is_vector_intrinsic(intrinsic) =>
                        {
                            // Merge input vector with output,
                            // Add the elements to the vector's pointee set
                            self.unify_vector_intrinsic(function, intrinsic, arguments, results);
                        }
                        (Value::Intrinsic(intrinsic), _) => {
                            // Only Hint or Vector operations may alias.
                            assert!(!Self::intrinsic_may_alias(intrinsic));
                        }
                        // Fallthrough for unresolved functions whose function body
                        // is not available, via a conservative type-based analysis.
                        _ => self.unresolved_call(function, arguments, results),
                    }
                }
                Instruction::ArrayGet { array, .. } => {
                    // Field-insensitive: array's pointee is merged with the elements (and the index is not used)
                    // Note that composite arrays can hold different types, which obviously cannot alias.
                    // So merging the elements is sound but imprecise.
                    let result = function.dfg.instruction_result::<1>(*instruction_id)[0];
                    self.merge_reference(function, result, *array);
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
                    let array = function.dfg.instruction_result::<1>(*instruction_id)[0];
                    for element in elements {
                        // Field-insensitive: each element joins new_array's pointee class
                        self.merge_reference(function, *element, array);
                    }
                }
                Instruction::IfElse { then_value, else_value, .. } => {
                    let result = function.dfg.instruction_result::<1>(*instruction_id)[0];
                    let typ = function.dfg.type_of_value(*then_value);
                    if typ.contains_reference() {
                        let result = GlobalValueId::new(function, result);
                        let then_value = GlobalValueId::new(function, *then_value);
                        let else_value = GlobalValueId::new(function, *else_value);
                        self.merge_alias(then_value, result);
                        self.merge_alias(else_value, result);
                        let allocation =
                            self.get_allocation(then_value).join(self.get_allocation(else_value));
                        if let AllocationLattice::Known(site) = allocation {
                            self.allocation_sites.insert(result, site);
                        }
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
    /// return_values (or create them the first time).
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
                let typ = canonicalize_type(&typ);
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

    /// Helper function for unresolved call which put identical types into buckets and merge their corresponding ValueId.
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
                self.unify_vector_op(function, arguments[1], results[1], &arguments[2..]);
            }
            Intrinsic::VectorInsert => {
                self.unify_vector_op(function, arguments[1], results[1], &arguments[3..]);
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

    /// Collect the result ValueIds of every `Allocate` instruction in the main
    /// function, in declaration order (across reachable blocks).
    fn collect_allocates(ssa: &Ssa) -> Vec<ValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Allocate) {
                    out.push(func.dfg.instruction_result::<1>(*inst_id)[0]);
                }
            }
        }
        out
    }

    /// Collect the result ValueIds of every `Load` instruction.
    fn collect_loads(ssa: &Ssa) -> Vec<ValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::Load { .. }) {
                    out.push(func.dfg.instruction_result::<1>(*inst_id)[0]);
                }
            }
        }
        out
    }

    /// Collect the result ValueIds of every `ArrayGet` instruction.
    fn collect_array_gets(ssa: &Ssa) -> Vec<ValueId> {
        let func = ssa.main();
        let mut out = Vec::new();
        for block_id in func.reachable_blocks() {
            for inst_id in func.dfg[block_id].instructions() {
                if matches!(&func.dfg[*inst_id], Instruction::ArrayGet { .. }) {
                    out.push(func.dfg.instruction_result::<1>(*inst_id)[0]);
                }
            }
        }
        out
    }

    fn collect_call_results_in_main(ssa: &Ssa) -> Vec<ValueId> {
        let main = ssa.main();
        let mut out = Vec::new();
        for block in main.reachable_blocks() {
            for inst_id in main.dfg[block].instructions() {
                if matches!(main.dfg[*inst_id], Instruction::Call { .. }) {
                    for result in main.dfg.instruction_results(*inst_id) {
                        out.push(*result);
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
        assert!(!analysis.is_aliased(ssa.main(), allocs[0]));
        assert!(!analysis.is_aliased(ssa.main(), allocs[1]));
    }

    /// `&T` and `&mut T` can legitimately point to the same memory at runtime.
    /// Types are canonicalized throughout the analysis so the `may_alias` type guard
    /// treats them as equivalent.
    #[test]
    fn mutable_and_immutable_refs_are_treated_as_same_type() {
        // Use oracles so v0 and v1 carry to avoid allocation-site.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_mut() -> &mut Field
            v1 = call oracle_ref() -> &Field
            call print(v0, v1)
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
        assert!(analysis.is_aliased(ssa.main(), call_results[0]));
        assert!(analysis.is_aliased(ssa.main(), call_results[1]));
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
        let v1 = func.dfg[b1].parameters()[0];
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(ssa.main(), v0, v1));
    }

    // ============================================================
    // OTHER CASES - more complex cases
    // ============================================================

    /// IfElse on composites merges the composite branches so
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
            result.unwrap()
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
            result.unwrap()
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

    /// vector_push_back is handled precisely by `unify_vector_intrinsic`:
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

    /// `vector_push_front` symmetric to push_back: pushed element lands in the
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
            result.unwrap()
        };
        let mut analysis = analyze_main(&ssa);
        // popped aliases v0 (originally in the vector's pointee class).
        assert!(analysis.may_alias(main, allocs[0], popped));
        // And v1 (also in the pointee class, field-insensitively).
        assert!(analysis.may_alias(main, allocs[1], popped));
    }

    /// `vector_pop_front`: layout differs (popped element comes before
    /// new_len and new_vec in the results list). Same aliasing effect.
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
            result.unwrap()
        };
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(main, allocs[0], popped));
        assert!(analysis.may_alias(main, allocs[1], popped));
    }

    /// `vector_insert`: inserted element lands in the pointee class, exactly
    /// like push_back/push_front but at an arbitrary index.
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
            result.unwrap()
        };
        let mut analysis = analyze_main(&ssa);
        assert!(analysis.may_alias(main, allocs[0], removed));
        assert!(analysis.may_alias(main, allocs[1], removed));
    }

    // ============================================================
    // `unresolved_call` — conservative handling of opaque calls
    // (foreign functions / unresolved function pointers).
    // Tests exercise the four cases of the cascade:
    //   1. Same type         → merge_alias
    //   2. Containment       → merge_reference
    //   3. Shared inner ref  → merge_alias
    //   4. Complex relation  → escape
    //
    // Uses `call print(...)` which the SSA parser resolves to a
    // ForeignFunction call, routed through `unify_on_signature`.
    // ============================================================

    /// Case 1: two args of the same ref type are merged via `merge_alias` —
    /// the callee might return one of them, or store one into the other's
    /// pointee.
    #[test]
    fn opaque_call_merges_same_typed_ref_args() {
        // Oracles so v0 and v1 start in separate classes.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            call print(v0, v1)
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
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = make_array [v1] : [&mut Field; 1]
            call print(v0, v2)
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
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field; 1]
            v3 = make_array [v1, v1] : [&mut Field; 2]
            call print(v2, v3)
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
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            v2 = allocate -> &mut u32
            call print(v0, v2)
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
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut [Field; 2]
            v2 = allocate -> &mut Field
            v3 = allocate -> &mut u32
            call print(v0, v1)
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
        assert!(analysis.is_aliased(ssa.main(), allocs[0]));
        assert!(analysis.is_aliased(ssa.main(), allocs[1]));
        // v2 wasn't in any call → not aliased.
        assert!(!analysis.is_aliased(ssa.main(), allocs[2]));
    }

    /// `type_representatives` collapses N entries of the same type into a single
    /// alias class via one `merge_alias` per additional entry — no pairwise loop.
    /// This test exercises N=4 to confirm the bucket merge reaches every entry,
    /// not just the first pair.
    #[test]
    fn unresolved_call_buckets_n_same_typed_refs_into_one_class() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call oracle_a() -> &mut Field
            v1 = call oracle_b() -> &mut Field
            v2 = call oracle_c() -> &mut Field
            v3 = call oracle_d() -> &mut Field
            call print(v0, v1, v2, v3)
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
          b0():
            v0 = call oracle_inner_a() -> &mut Field
            v1 = call oracle_inner_b() -> &mut Field
            v2 = make_array [v0] : [&mut Field; 1]
            call print(v0, v1, v2)
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
          b0():
            v0 = call oracle_inner_a() -> &mut Field
            v1 = make_array [v0] : [&mut Field; 1]
            call print(v0, v1)
            v2 = call oracle_inner_b() -> &mut Field
            v3 = make_array [v2] : [&mut Field; 1]
            call print(v3, v2)
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
    /// the ArraySet handler are no-ops, and neither the global nor the
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
}
