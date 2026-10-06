//! This module defines the [`Ssa::check_for_missing_brillig_constraints`] method.
//!
//! It verifies that the output of Brillig calls is connected to the inputs of the calls
//! by assertions; in other words, that the circuit has constraints that the output is
//! correct, given the inputs.
//!
//! To do so, it tracks the ancestry of every expression, and checks that any
//! variable which is an output of a Brillig call has a descendant which appears
//! in an assertion, where the other side has an ancestor that is an input of the call.
//!
//! Essentially, to consider a particular Brillig call constrained, we are looking
//! for a constraint where the ancestors of the constraint arguments intersect both of:
//! * the descendants of the results of the call (outputs)
//! * the ancestors of the arguments of the call (inputs)
//!
//! For example take the following graph of variables feeding into calls:
//! ```text
//!   v1     v2      v3
//!    \   /  \    /
//!     \ /    \  /
//!      v4     v5 = call(v2, v3)
//!      |\     |
//!      | \    |
//!      |  \   |
//!      |   \  |
//!      |    \ |
//!      |      v6 = call(v5, v4)
//!      |     /
//!      |    /
//!      |   /
//!      |  /
//! constrain(v4, v6)
//! ```
//!
//! Both calls are considered constrained:
//! * The output of the 2nd call (v6) is constrained directly against its input (v4)
//! * The output of the 1st call (v5) has a descendant (v6) which is constrained against
//!   a value (v4) that has an ancestor (v2) which is also an ancestor of an argument of
//!   of the call itself.
//!
//! The goal isn't to verify that the constraint is correct, just that some (indirect)
//! connection between inputs and outputs is made.
use crate::ssa::checks::is_numeric_constant;
use crate::ssa::ir::basic_block::BasicBlockId;
use crate::ssa::ir::dfg::DataFlowGraph;
use crate::ssa::ir::function::{Function, FunctionId};
use crate::ssa::ir::instruction::{Instruction, InstructionId, Intrinsic};
use crate::ssa::ir::post_order::PostOrder;
use crate::ssa::ir::value::{Value, ValueId};
use crate::ssa::ssa_gen::Ssa;
use acvm::AcirField;
use bit_vec::BitVec;
use iter_extended::vecmap;
use noirc_artifacts::ssa::{InternalBug, SsaReport};
use rayon::prelude::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::cmp;
use std::collections::{BTreeMap, VecDeque};

/// The maximum length of arrays that we attempt to constrain item-by-item.
///
/// Arrays longer than this value will be considered constrained if any item
/// we get from them gets constrained.
///
/// The higher this value the longer it will take to check them all,
/// which can slow down the compilation of larger rollup circuits.
pub const DEFAULT_MAX_ARRAY_OUTPUT_LENGTH: u32 = 64;

/// Limit how far back the BFS traverses to try to find a relation between
/// the ancestors of constrained values and Brillig inputs/outputs.
///
/// This exists to help keep the runtime down on the largest protocol circuits,
/// such as `rollup-checkpoint-root` and `rollup-checkpoint-root-single-block`,
/// which have hundreds of thousands of constraints that we need to check,
/// even though they are only checked against a few dozen of Brillig outputs.
pub const DEFAULT_MAX_ANCESTOR_DISTANCE: u32 = 10;

impl Ssa {
    /// Detect Brillig calls left unconstrained with manual asserts
    /// and return a vector of bug reports if any have been found
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn check_for_missing_brillig_constraints(
        &mut self,
        max_array_output_length: u32,
        max_ancestor_distance: u32,
    ) -> Vec<SsaReport> {
        // Skip the check if there are no Brillig functions involved
        if !self.functions.values().any(|func| func.runtime().is_brillig()) {
            return vec![];
        }

        self.functions
            .values()
            .filter(|func| func.runtime().is_acir() && has_call_to_brillig(func, &self.functions))
            .par_bridge()
            .flat_map(|func| {
                Context::new(func, max_array_output_length, max_ancestor_distance)
                    .build_tainted(func, &self.functions)
                    .build_parent_graph(func)
                    .constrain_tainted(func, &self.functions)
                    .into_warnings(func)
            })
            .collect()
    }
}

/// A more compact representation of a `HashSet<ValueId>` to limit memory use.
#[derive(Debug)]
struct ValueSet(BitVec<u32>);

impl ValueSet {
    fn new(dfg: &DataFlowGraph) -> Self {
        Self(BitVec::from_elem(dfg.num_values(), false))
    }

    fn contains(&self, value: &ValueId) -> bool {
        self.0.get(value.to_u32().try_into().unwrap()).expect("initialized with all values")
    }

    fn insert(&mut self, value: ValueId) {
        self.0.set(value.to_u32().try_into().unwrap(), true);
    }

    fn extend<'a>(&mut self, values: impl IntoIterator<Item = &'a ValueId>) {
        for value in values {
            self.insert(*value);
        }
    }
}

/// Position of a tainted Brillig call in [`Context::tainted`].
type TaintedIndex = usize;

/// A growable bitset of [`TaintedIndex`]es.
///
/// Used to record, for each value, which tainted calls it is relevant to, so that
/// propagating that relevance through an instruction costs one word per 64 calls
/// instead of a visit to every call.
#[derive(Debug, Default, Clone)]
struct TaintedSet(Vec<u64>);

impl TaintedSet {
    fn insert(&mut self, index: TaintedIndex) {
        let word = index / 64;
        if word >= self.0.len() {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1 << (index % 64);
    }

    fn remove(&mut self, index: TaintedIndex) {
        if let Some(word) = self.0.get_mut(index / 64) {
            *word &= !(1 << (index % 64));
        }
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }

    fn union_with(&mut self, other: &TaintedSet) {
        if other.0.len() > self.0.len() {
            self.0.resize(other.0.len(), 0);
        }
        for (word, other) in self.0.iter_mut().zip(&other.0) {
            *word |= other;
        }
    }

    fn intersect_with(&mut self, other: &TaintedSet) {
        self.0.truncate(other.0.len());
        for (word, other) in self.0.iter_mut().zip(&other.0) {
            *word &= other;
        }
    }

    /// Iterate the members in ascending order.
    fn iter(&self) -> impl Iterator<Item = TaintedIndex> + '_ {
        self.0.iter().enumerate().flat_map(|(i, word)| {
            let mut word = *word;
            std::iter::from_fn(move || {
                if word == 0 {
                    return None;
                }
                let bit = word.trailing_zeros() as usize;
                word &= word - 1;
                Some(i * 64 + bit)
            })
        })
    }
}

/// Direct parents and equivalences of tracked values, through which ancestry is traversed.
///
/// Transitive ancestry is computed on demand via BFS instead of being pre-computed.
#[derive(Debug, Default)]
struct AncestryGraph {
    /// Direct parent graph for tracked values.
    ///
    /// `parents[v]` = the immediate instruction arguments that produced `v`,
    /// plus the active side-effect condition (if any) at the time `v` was produced.
    ///
    /// We track parents for values which either:
    /// * have constraints on them, or
    /// * are inputs to a Brillig call.
    parents: HashMap<ValueId, Vec<ValueId>>,

    /// Bidirectional equivalence edges from `constrain v1 == v2` instructions.
    ///
    /// If `v1` and `v2` are equivalent, any ancestor of `v1` is also an ancestor of `v2`
    /// and vice versa. BFS follows these edges alongside `parents` edges.
    equivalences: HashMap<ValueId, Vec<ValueId>>,

    /// The array of each tracked value read from an array at a dynamic index.
    ///
    /// The array is not a parent of such a value (see [`parent_arguments`]), so that a
    /// constraint on it does not appear to constrain every item of the array. The value is
    /// still derived from the array, though: if the array is constrained, so is the value.
    read_arrays: HashMap<ValueId, ValueId>,
}

impl AncestryGraph {
    /// Whether we are collecting the parents of a value.
    fn is_tracked(&self, value: &ValueId) -> bool {
        self.parents.contains_key(value)
    }

    /// Start collecting the parents of a value.
    fn track(&mut self, value: ValueId) {
        self.parents.entry(value).or_default();
    }

    /// Add direct parents to a value, and start tracking the parents themselves, so that
    /// when we reach the instructions producing them (going backward), we expand their
    /// parents too.
    fn add_parents(&mut self, value: ValueId, parents: &[ValueId]) {
        self.parents.entry(value).or_default().extend(parents.iter().copied());
        for parent in parents {
            self.track(*parent);
        }
    }

    /// Remove `old` from the parents of a value, and add `new`, if any, in its place.
    fn replace_parent(&mut self, value: ValueId, old: ValueId, new: Option<ValueId>) {
        let parents = self.parents.get_mut(&value).expect("value should be tracked");
        parents.retain(|parent| *parent != old);
        if let Some(new) = new {
            parents.push(new);
            self.track(new);
        }
    }

    /// Record that `v1` and `v2` are constrained to be equal.
    fn add_equivalence(&mut self, v1: ValueId, v2: ValueId) {
        self.equivalences.entry(v1).or_default().push(v2);
        self.equivalences.entry(v2).or_default().push(v1);
    }

    /// Record that `value` was read from `array` at a dynamic index, and start tracking the array.
    fn add_read_array(&mut self, value: ValueId, array: ValueId) {
        self.read_arrays.insert(value, array);
        self.track(array);
    }

    /// Traverse the values reachable (inclusive) from any of the `starts` by following
    /// `parents` and `equivalences` edges backwards, breadth first.
    ///
    /// Equivalences are only followed from **intermediate** nodes (not from the starting nodes
    /// themselves). This matches the original transitive-closure semantics: `constrain v1 == v2`
    /// adds v2 to the ancestor sets of keys that *already* have v1 as an ancestor, but does **not**
    /// add v2 to v1's own ancestor set (because v1 is never its own ancestor).
    ///
    /// Calls a function `f` with each value and its distance; if `f` returns `true` the
    /// traversal continues, otherwise returns.
    ///
    /// Returns the set of visited nodes.
    fn traverse(
        &self,
        starts: &[ValueId],
        f: impl FnMut(ValueId, u32) -> bool,
    ) -> HashSet<ValueId> {
        self.traverse_with(starts, false, f)
    }

    /// Like [`Self::traverse`], but if `follow_read_arrays` is set, it also goes from values
    /// read at a dynamic index to the array they were read from.
    fn traverse_with(
        &self,
        starts: &[ValueId],
        follow_read_arrays: bool,
        mut f: impl FnMut(ValueId, u32) -> bool,
    ) -> HashSet<ValueId> {
        let read_array =
            |value: &ValueId| follow_read_arrays.then(|| self.read_arrays.get(value)).flatten();
        let mut visited: HashSet<ValueId> = HashSet::default();
        let mut queue: VecDeque<(ValueId, u32)> = VecDeque::new();
        for &s in starts {
            visited.insert(s);
            if !f(s, 0) {
                return visited;
            }
            // From start nodes: follow only parent edges, not equivalences.
            for &p in self.parents.get(&s).into_iter().flatten().chain(read_array(&s)) {
                if visited.insert(p) {
                    queue.push_back((p, 1));
                }
            }
        }
        // From intermediate nodes: follow both parent and equivalence edges.
        while let Some((curr, dist)) = queue.pop_front() {
            if !f(curr, dist) {
                return visited;
            }
            for &next in self
                .parents
                .get(&curr)
                .into_iter()
                .flatten()
                .chain(self.equivalences.get(&curr).into_iter().flatten())
                .chain(read_array(&curr))
            {
                if visited.insert(next) {
                    queue.push_back((next, dist + 1));
                }
            }
        }
        visited
    }

    /// Compute the set of all values reachable (inclusive) from any of the `starts` by following
    /// `parents` and `equivalences` edges backwards.
    fn ancestors(&self, starts: &[ValueId]) -> HashSet<ValueId> {
        self.traverse(starts, |_, _| true)
    }

    /// The [`Ball`] around a value.
    fn ball(&self, start: ValueId, max_ancestor_distance: u32) -> Ball {
        let collect = |follow_read_arrays| {
            let mut values = Vec::new();
            self.traverse_with(&[start], follow_read_arrays, |a, d| {
                values.push(a);
                d <= max_ancestor_distance
            });
            values
        };
        let values = collect(false);
        let set = values.iter().copied().collect();
        let data_values = if self.read_arrays.is_empty() { values.clone() } else { collect(true) };
        Ball { values, set, data_values }
    }
}

/// The values within `max_ancestor_distance` of a constrained value, found by following
/// `parents` (and `equivalences` from intermediate nodes) backwards. The traversal also
/// includes the first value it reaches beyond that distance.
///
/// A constrained value is checked against every Brillig call it may be relevant to,
/// each asking several questions about its ancestry ("is this output an ancestor?",
/// "is an ancestor in `arg_ancestors`?"). Computing the ball once per constrained value
/// turns each of those questions into lookups instead of separate traversals.
#[derive(Debug)]
struct Ball {
    values: Vec<ValueId>,
    set: HashSet<ValueId>,
    /// The values within the same distance when also going from values read at a dynamic
    /// index to their array. Used to tell whether a tainted value has been constrained.
    data_values: Vec<ValueId>,
}

impl Ball {
    /// Whether `value` is in the ball.
    fn contains(&self, value: &ValueId) -> bool {
        self.set.contains(value)
    }

    /// Whether any value in the ball satisfies the predicate.
    fn any(&self, predicate: impl Fn(&ValueId) -> bool) -> bool {
        self.values.iter().any(predicate)
    }
}

/// Outputs of a Brillig call and their descendants.
#[derive(Debug)]
struct TaintedDescendants {
    /// The call instruction.
    instruction_id: InstructionId,
    /// Inputs of the call.
    ///
    /// To consider the call constrained, the constraint must be on a value which has
    /// an ancestry that intersects with the ancestry of an argument.
    arguments: Vec<ValueId>,
    /// Non-array outputs of the call.
    ///
    /// To consider the output constrained, we have to find a constraint such that
    /// the output is an ancestor of the constrained value.
    single_outputs: HashSet<ValueId>,
    /// Array outputs of the call, tracked per index, accumulating their individual
    /// dependencies (only the values read from the array).
    ///
    /// To consider an element constrained, we have to find a constraint such that
    /// the constrained value appears in the descendants.
    array_outputs: HashMap<ValueId, HashMap<u32, HashSet<ValueId>>>,
    /// The union of all values reachable from any argument by following parents and
    /// equivalences backwards. Includes the arguments themselves.
    ///
    /// Pre-computed after the parent graph is built so that `arguments_intersect`
    /// can check membership in O(1) rather than re-running BFS for every constraint.
    arg_ancestors: HashSet<ValueId>,
}

impl TaintedDescendants {
    /// Create a new `TaintedDescendants` from the arguments and results of a call.
    ///
    /// Populates `single_outputs` and `array_outputs` according to the result types.
    /// Leaves `arg_ancestors` to be populated later.
    fn new(
        func: &Function,
        instruction_id: InstructionId,
        arguments: Vec<ValueId>,
        result_ids: &[ValueId],
        max_array_output_length: u32,
    ) -> Self {
        let mut single_outputs = HashSet::default();
        let mut array_outputs = HashMap::default();
        for result_id in result_ids {
            match func.dfg.try_get_array_length(*result_id) {
                // If the result value is an array, create an empty descendant set for
                // every element to be accessed further on and record the indices
                // of the resulting sets for future reference
                Some(length) if length.0 > 0 && length.0 <= max_array_output_length => {
                    let mut index_outputs = HashMap::default();
                    for i in 0..length.0 {
                        index_outputs.insert(i, HashSet::default());
                    }
                    array_outputs.insert(*result_id, index_outputs);
                }
                // For very large arrays or non-arrays, treat the whole result as a single value
                // to avoid memory/time issues when tracking individual elements
                Some(_) | None => {
                    single_outputs.insert(*result_id);
                }
            }
        }

        Self {
            instruction_id,
            arguments,
            single_outputs,
            array_outputs,
            arg_ancestors: HashSet::default(),
        }
    }

    /// Whether there are any unconstrained outputs left.
    /// Returns `true` if the call is fully constrained.
    fn is_fully_constrained(&self) -> bool {
        self.single_outputs.is_empty() && self.array_outputs.is_empty()
    }

    /// Try to constrain some of the outputs if:
    /// * one of the constrained values is a descendant of the output, and
    /// * another constrained value shares an ancestor with an input,
    ///   and it is not tainted, or it has been already constrained
    ///
    /// Exceptions to this rule are:
    /// * if there are no input arguments (they were all numeric constants, or there were no args)
    /// * if there is only one constrained value (an output against a constant)
    ///
    /// The caller is expected to only pass constraints which are relevant to this call,
    /// ie. ones where at least one of the constrained values is constrainable for it.
    ///
    /// `balls` holds the [`Ball`] of each of the `constrained_values`, in the same order.
    ///
    /// Any constrained output is added to the `all_constrained` set.
    ///
    /// Returns `true` if at least one output was cleared by this call. Each output is
    /// cleared at most once (it is removed from its set when cleared), so this reflects
    /// genuinely new progress, which the fixed-point loop in [`Context::constrain_tainted`]
    /// uses to decide whether another walk is worthwhile.
    fn try_constrain(
        &mut self,
        constrained_values: &[ValueId],
        balls: &[Ball],
        all_tainted: &ValueSet,
        all_constrained: &mut ValueSet,
    ) -> bool {
        let is_against_const = constrained_values.len() == 1;
        let is_const_args = self.arguments.is_empty();

        // Make sure this constraint has something to do with the inputs,
        // unless there are no inputs, or the output is against a constant.
        if !is_against_const
            && !is_const_args
            && !self.arguments_intersect(constrained_values, balls, all_tainted, all_constrained)
        {
            return false;
        }

        // Set whenever an output is cleared below.
        let mut progressed = false;

        // Remove any results that have been directly or indirectly constrained.
        self.single_outputs.retain(|output| {
            let constrained = balls.iter().any(|ball| ball.contains(output));

            if constrained {
                all_constrained.insert(*output);
                progressed = true;
            }

            !constrained
        });

        self.array_outputs.retain(|array, index_outputs| {
            // If the array itself is not an ancestor of the constrained value, then we don't have to check the items.
            let can_constrain = balls.iter().any(|ball| ball.contains(array));

            if !can_constrain {
                return true;
            }

            // Remove whichever index was constrained.
            index_outputs.retain(|_index, descendants| {
                // Until we have seen an ArrayGet and know which value is the output,
                // we can't tell this index has been constrained.
                if descendants.is_empty() {
                    return true;
                }
                let constrained =
                    balls.iter().any(|ball| descendants.iter().any(|value| ball.contains(value)));

                if constrained {
                    all_constrained.extend(descendants.iter());
                    progressed = true;
                }

                !constrained
            });

            // Keep the array until all indexed items have been constrained.
            if index_outputs.is_empty() {
                // Once all its items are constrained, the array as a whole is constrained too,
                // which matters when it is passed on whole, for example into another call.
                all_constrained.insert(*array);
                false
            } else {
                true
            }
        });

        progressed
    }

    /// Whether one of the constrained values:
    /// * shares an ancestor with a call argument (checked via pre-computed `arg_ancestors`), and
    /// * is not tainted, unless it's been already constrained
    fn arguments_intersect(
        &self,
        constrained_values: &[ValueId],
        balls: &[Ball],
        all_tainted: &ValueSet,
        all_constrained: &ValueSet,
    ) -> bool {
        for (cv, ball) in constrained_values.iter().zip(balls) {
            // We want to avoid using tainted inputs to constrain Brillig outputs.
            // Allowing them would mean we could constrain the output of one call
            // with the output of another Brillig call, and also that outputs of
            // the call would trivially connect to the inputs.
            // However if a tainted input has been constrained already, we can use it.
            if all_tainted.contains(cv)
                && (
                    // Tainted and hasn't been constrained.
                    !ball.data_values.iter().any(|a| all_constrained.contains(a))
                    // Tainted because it's the output of this call itself.
                    || self.single_outputs.iter().any(|output| ball.contains(output))
                    || self.array_outputs.keys().any(|array| ball.contains(array))
                )
            {
                continue;
            }
            // arg_ancestors contains the arguments themselves and all their transitive ancestors.
            // Check if cv or any ancestor of cv is in arg_ancestors.
            if ball.any(|a| self.arg_ancestors.contains(a)) {
                return true;
            }
        }
        false
    }

    /// Add to the descendants of a particular array element.
    ///
    /// This is only called when we read from an array. Later on we can use the
    /// ancestry information to connect constrained values back to values we read
    /// from the array.
    fn extend_array_result(&mut self, array: ValueId, index: u32, results: &[ValueId]) {
        let Some(index_outputs) = self.array_outputs.get_mut(&array) else {
            return;
        };
        let Some(descendants) = index_outputs.get_mut(&index) else {
            return;
        };
        descendants.extend(results);
    }
}

/// The instructions that [`Context::constrain_tainted`] needs to visit, in Reverse Post Order.
#[derive(Debug)]
enum Event {
    /// A tainted Brillig call, after which constraints on its outputs can be considered.
    Call(TaintedIndex),
    /// A relevant constraint, with its non-constant arguments.
    Constraint(Vec<ValueId>),
}

/// Values which are worth looking for constraints on, because they are the outputs of
/// tainted calls, or descend from them within the ancestor distance.
///
/// This helps eliminate constraints which are of no effect.
#[derive(Debug, Default)]
struct Constrainable(HashMap<ValueId, ConstrainableValue>);

#[derive(Debug)]
struct ConstrainableValue {
    /// Distance from the outputs of the tainted calls.
    distance: u32,
    /// The tainted calls for which constraints on the value are interesting.
    owners: TaintedSet,
}

impl Constrainable {
    fn contains(&self, value: &ValueId) -> bool {
        self.0.contains_key(value)
    }

    /// Track the outputs of a tainted call.
    fn insert_outputs(&mut self, index: TaintedIndex, outputs: &[ValueId]) {
        let mut owners = TaintedSet::default();
        owners.insert(index);
        for output in outputs {
            self.set(*output, 0, &owners);
        }
    }

    /// Track the `results` of an instruction as descendants of any constrainable `args`,
    /// unless that would take them beyond `max_distance`.
    fn extend(&mut self, args: &[ValueId], results: &[ValueId], max_distance: u32) {
        let mut min_distance: Option<u32> = None;
        let mut owners = TaintedSet::default();
        for arg in args {
            if let Some(value) = self.0.get(arg) {
                min_distance =
                    Some(min_distance.map_or(value.distance, |d| cmp::min(d, value.distance)));
                owners.union_with(&value.owners);
            }
        }
        if let Some(distance) = min_distance
            && distance < max_distance
        {
            for result in results {
                self.set(*result, distance + 1, &owners);
            }
        }
    }

    /// If `from` is constrainable, then make `to` constrainable at the same distance,
    /// for the same calls.
    ///
    /// Returns whether `from` was constrainable.
    fn alias(&mut self, from: ValueId, to: ValueId) -> bool {
        let Some(value) = self.0.get(&from) else {
            return false;
        };
        let distance = value.distance;
        let owners = value.owners.clone();
        self.set(to, distance, &owners);
        true
    }

    /// The tainted calls for which constraints on any of the `values` are interesting.
    fn owners_of(&self, values: &[ValueId]) -> TaintedSet {
        let mut owners = TaintedSet::default();
        for value in values {
            if let Some(value) = self.0.get(value) {
                owners.union_with(&value.owners);
            }
        }
        owners
    }

    /// Set the distance of a value, and add to the calls it is interesting for.
    fn set(&mut self, value: ValueId, distance: u32, owners: &TaintedSet) {
        let entry = self
            .0
            .entry(value)
            .or_insert_with(|| ConstrainableValue { distance, owners: TaintedSet::default() });
        entry.distance = distance;
        entry.owners.union_with(owners);
    }
}

/// The tainted Brillig calls, addressed by [`TaintedIndex`].
#[derive(Debug, Default)]
struct TaintedCalls {
    /// Descendants of Brillig calls, in the order the calls were encountered.
    calls: Vec<TaintedDescendants>,

    /// Index of each call by its instruction.
    by_instruction: HashMap<InstructionId, TaintedIndex>,

    /// The call that each tracked array output belongs to.
    array_output_owner: HashMap<ValueId, TaintedIndex>,

    /// Calls which still have unconstrained outputs.
    unresolved: TaintedSet,
}

impl TaintedCalls {
    /// Register a call, returning its index.
    fn push(&mut self, tainted: TaintedDescendants) -> TaintedIndex {
        let index = self.calls.len();
        for array in tainted.array_outputs.keys() {
            self.array_output_owner.insert(*array, index);
        }
        self.by_instruction.insert(tainted.instruction_id, index);
        self.unresolved.insert(index);
        self.calls.push(tainted);
        index
    }

    fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    /// The index of the call made by an instruction, if it is tainted.
    fn index_of(&self, instruction_id: &InstructionId) -> Option<TaintedIndex> {
        self.by_instruction.get(instruction_id).copied()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut TaintedDescendants> {
        self.calls.iter_mut()
    }

    /// Add to the descendants of an element of an array output, if the array is tracked.
    fn extend_array_result(&mut self, array: ValueId, index: u32, results: &[ValueId]) {
        if let Some(owner) = self.array_output_owner.get(&array) {
            self.calls[*owner].extend_array_result(array, index, results);
        }
    }

    /// Calls which still have unconstrained outputs.
    fn unresolved(&self) -> &TaintedSet {
        &self.unresolved
    }

    /// Try to constrain the outputs of a call with a constraint, marking the call
    /// resolved once all its outputs are constrained. See [`TaintedDescendants::try_constrain`].
    fn try_constrain(
        &mut self,
        index: TaintedIndex,
        constrained_values: &[ValueId],
        balls: &[Ball],
        all_tainted: &ValueSet,
        all_constrained: &mut ValueSet,
    ) -> bool {
        let tainted = &mut self.calls[index];
        let progressed =
            tainted.try_constrain(constrained_values, balls, all_tainted, all_constrained);
        if tainted.is_fully_constrained() {
            self.unresolved.remove(index);
        }
        progressed
    }

    /// The instructions of the calls which still have unconstrained outputs.
    fn unresolved_instructions(&self) -> impl Iterator<Item = InstructionId> + '_ {
        self.unresolved.iter().map(|index| self.calls[index].instruction_id)
    }
}

#[derive(Debug)]
struct Context {
    /// Block IDs in Post Order.
    post_order: Vec<BasicBlockId>,

    /// Brillig calls whose outputs need to be constrained.
    tainted: TaintedCalls,

    /// Values which are worth looking for constraints on, with the calls they are relevant to.
    constrainable: Constrainable,

    /// Constraints which will be relevant to constraining Brillig outputs.
    ///
    /// These are determined during the initial top-down pass,
    /// so that we can limit the amount of ancestry we collect.
    constraints: HashSet<InstructionId>,

    /// Ancestry of the values relevant to constraining Brillig outputs.
    graph: AncestryGraph,

    /// Maximum length of an array for which we consider constraining items per index.
    max_array_output_length: u32,

    /// Maximum distance to travel looking for an intersecting ancestor.
    max_ancestor_distance: u32,
}

impl Context {
    fn new(func: &Function, max_array_output_length: u32, max_ancestor_distance: u32) -> Self {
        Self {
            post_order: PostOrder::with_function(func).into_vec(),
            tainted: TaintedCalls::default(),
            constrainable: Constrainable::default(),
            constraints: HashSet::default(),
            graph: AncestryGraph::default(),
            max_array_output_length,
            max_ancestor_distance,
        }
    }

    /// Build a direct parent graph for tracked values, then compute `arg_ancestors` for each
    /// tainted Brillig call via BFS.
    ///
    /// This avoids having to have a transitive-closure `ancestors` map, with a compact representation:
    /// `parents[v]` stores only the immediate instruction arguments of `v` (plus the active
    /// side-effect condition, if any). Transitive ancestry is computed on demand during BFS.
    fn build_parent_graph(mut self, func: &Function) -> Self {
        // Forward sub-pass: collect which side-effect condition (if any) is active at each
        // instruction, so we can add it as a parent during the backward pass below.
        let mut side_effect_at: HashMap<InstructionId, ValueId> = HashMap::default();
        for block_id in self.post_order.iter().copied().rev() {
            let mut current_se: Option<ValueId> = None;
            for instr_id in func.dfg[block_id].instructions() {
                if let Instruction::EnableSideEffectsIf { condition } = &func.dfg[*instr_id] {
                    current_se = (!is_numeric_constant(func, *condition)).then_some(*condition);
                } else if let Some(se) = current_se {
                    side_effect_at.insert(*instr_id, se);
                }
            }
        }

        // Backward pass: build the parent graph.
        //
        // pending_loads[address] = list of tracked load results whose direct parent is `address`.
        // When we later encounter Store { address, value }, we fix those parents up.
        let mut pending_loads: HashMap<ValueId, Vec<ValueId>> = HashMap::default();

        for block_id in self.post_order.iter().copied() {
            for instruction_id in func.dfg[block_id].instructions().iter().rev() {
                let instruction = &func.dfg[*instruction_id];
                let result_ids = func.dfg.instruction_results(*instruction_id);

                // For each tracked result, add its instruction's arguments as direct parents.
                // Compute args lazily — only when we find a tracked result.
                let mut args: Option<Vec<ValueId>> = None;

                for result_id in result_ids {
                    if is_numeric_constant(func, *result_id) || !self.graph.is_tracked(result_id) {
                        continue;
                    }

                    let args = args.get_or_insert_with(|| parent_arguments(func, instruction));

                    self.graph.add_parents(*result_id, args);

                    if let Instruction::ArrayGet { array, index } = instruction
                        && func.dfg.get_numeric_constant(*index).is_none()
                    {
                        self.graph.add_read_array(*result_id, *array);
                    }

                    // Add the active side-effect condition as an additional parent so that
                    // BFS can reach the condition's ancestors from this result.
                    if let Some(&se) = side_effect_at.get(instruction_id) {
                        self.graph.add_parents(*result_id, &[se]);
                    }

                    // If this is a Load, remember it so Store can fix up the placeholder parent.
                    // Note that by the time this pass runs, Loads and Stores should have been
                    // removed from ACIR functions, but we do have some unit tests that uses them.
                    // This could be removed, but we kept it in case things change in the future.
                    if let Instruction::Load { address } = instruction {
                        pending_loads.entry(*address).or_default().push(*result_id);
                    }
                }

                // Store resolution: replace the address placeholder with the stored value in
                // all pending load results for this address. By using remove(), only the
                // first Store encountered (going backward) resolves the loads.
                if let Instruction::Store { address, value } = instruction
                    && let Some(pending) = pending_loads.remove(address)
                {
                    let value = (!is_numeric_constant(func, *value)).then_some(*value);
                    for tracked in pending {
                        self.graph.replace_parent(tracked, *address, value);
                    }
                }

                // Start tracking the direct parents of this instruction's arguments if it is
                // a tainted call, a relevant constraint, or an EnableSideEffectsIf instruction.
                let should_track = self.tainted.index_of(instruction_id).is_some()
                    || self.constraints.contains(instruction_id)
                    || is_side_effect(func, instruction);

                if should_track {
                    let args = args.get_or_insert_with(|| instruction_arguments(func, instruction));
                    for value_id in args.iter() {
                        self.graph.track(*value_id);
                    }
                }

                // Collect equivalences from `constrain v1 == v2`.
                // These are followed bidirectionally during BFS so that ancestry flows
                // through equivalent values.
                if let Some((v1, v2)) = as_equivalence(func, instruction) {
                    self.graph.add_equivalence(v1, v2);
                }
            }
        }

        // BFS sub-pass: compute arg_ancestors for each tainted Brillig call.
        // arg_ancestors is the union of all values reachable backwards from any argument,
        // including the arguments themselves. This is pre-computed once so that
        // arguments_intersect can check membership in O(1) per constrained value.
        for tainted in self.tainted.iter_mut() {
            tainted.arg_ancestors = self.graph.ancestors(&tainted.arguments);
        }

        self
    }

    /// Traverse blocks and instructions top-down to build up the descendants of Brillig calls.
    fn build_tainted(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        // Traverse in Reverse Post Order, ie. top-down.
        for block_id in self.post_order.clone().into_iter().rev() {
            // Track the current side effect variable, unless it's a constant.
            let mut side_effects_var: Option<ValueId> = None;
            // No need to look for constraints on calls which originate from the same code location;
            // these are the result of unrolling loops, and it should be enough to cover the first.
            let mut visited_locations = HashSet::default();

            for instruction_id in func.dfg[block_id].instructions() {
                let instruction = &func.dfg[*instruction_id];
                let mut arguments = instruction_arguments(func, instruction);
                let results = instruction_results(func, instruction_id);

                // If we are under a side effect, extend the args.
                if let Some(side_effects_var) = &side_effects_var {
                    arguments.push(*side_effects_var);
                }

                // Extend the descendants of Brillig calls.
                // This is only required for array output; for single outputs we can look at the ancestry.
                if !results.is_empty() {
                    // Look for ArrayGet instructions with a constant index,
                    // and if the array is the result of a tainted call,
                    // then add the result as a descendant of that particular index.
                    if let Instruction::ArrayGet { array, index } = instruction
                        && let Some(index) = func.dfg.get_numeric_constant(*index)
                        && let Some(index) = index.try_to_u32()
                    {
                        self.tainted.extend_array_result(*array, index, &results);
                    }

                    // Extend the values we are looking to constrain, as long as we will
                    // not exceed the traversal limit to reach them.
                    self.constrainable.extend(&arguments, &results, self.max_ancestor_distance);
                }

                // If this is a Store instruction, then it has no result: instead if the value we store
                // is constrainable, then we can add the address to the constrainable set.
                // Keep the same distance as the address is just a handover point for values.
                if let Instruction::Store { address, value } = instruction {
                    self.constrainable.alias(*value, *address);
                }

                // If we have a constraint that means two values are equal, then we are interested
                // in constraints on the descendants on either of those, even if one of them is
                // not a descendant of Brillig outputs.
                if let Some((v1, v2)) = as_equivalence(func, instruction)
                    && !self.constrainable.alias(v1, v2)
                {
                    self.constrainable.alias(v2, v1);
                }

                if is_call_to_brillig(func, all_functions, instruction_id) && !results.is_empty() {
                    // Skip already visited locations (happens often in unrolled functions)
                    let call_stack = func.dfg.get_instruction_call_stack(*instruction_id);
                    let location = call_stack.last();

                    // If there is no call stack (happens for tests), consider unvisited
                    let visited = match location {
                        None => false,
                        Some(loc) if loc.is_dummy() => false,
                        Some(loc) => {
                            let Instruction::Call { func: callee, .. } = instruction else {
                                unreachable!("ICE: Expected Brillig call");
                            };
                            !visited_locations.insert((*callee, *loc))
                        }
                    };

                    // Skip if we have a similar one already.
                    if !visited {
                        let tainted = TaintedDescendants::new(
                            func,
                            *instruction_id,
                            arguments,
                            &results,
                            self.max_array_output_length,
                        );
                        let index = self.tainted.push(tainted);
                        // Look out for constraints on these outputs.
                        // We don't need to consider the inputs: the constraints which are relevant will have to constrain
                        // at least one output. Then, we will look at whether the other constrained value is related to
                        // the inputs, based on its ancestry, collected later for all inputs of relevant constraints.
                        self.constrainable.insert_outputs(index, &results);
                    }
                } else if is_constraint(func, instruction_id) && !self.tainted.is_empty() {
                    let constrained_values = instruction_arguments(func, instruction);
                    // If this constraint involves a Brillig output, then we can use it later, otherwise it's not interesting.
                    if constrained_values.iter().any(|value| self.constrainable.contains(value)) {
                        self.constraints.insert(*instruction_id);
                    }
                } else if let Instruction::EnableSideEffectsIf { condition } = instruction {
                    side_effects_var =
                        (!is_numeric_constant(func, *condition)).then_some(*condition);
                }
            }
        }

        self
    }

    /// Try to constrain Brillig outputs by visiting the relevant calls and constraints top-down.
    fn constrain_tainted(
        mut self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> Self {
        let (events, all_tainted) = self.collect_events(func, all_functions);

        // Persists across passes: an output shown to be constrained anywhere in the
        // function stays constrained, so a later constraint can rely on it regardless
        // of the source order of the two assertions. This is what makes the check
        // order-independent and requires iterating to a fixed point below.
        let mut all_constrained = ValueSet::new(&func.dfg);

        loop {
            let progressed =
                self.constrain_tainted_pass(&events, &all_tainted, &mut all_constrained);

            // Re-walk only while we are still making progress and work remains.
            // Fully constrained functions resolve every call in the first pass (no
            // extra walk), and genuinely under-constrained ones make no progress and stop.
            if self.tainted.unresolved().is_empty() || !progressed {
                break;
            }
        }

        self
    }

    /// Traverse blocks and instructions top-down to collect the tainted calls and relevant
    /// constraints in the order [`Self::constrain_tainted_pass`] visits them, along with
    /// the set of values which descend from any Brillig call.
    ///
    /// Constraints on tainted values cannot be used to connect output to input. Values are
    /// defined before they are used in Reverse Post Order, so whether a constrained value is
    /// tainted is already settled when the walk reaches its constraint, and the final set
    /// can be shared by every pass.
    fn collect_events(
        &self,
        func: &Function,
        all_functions: &BTreeMap<FunctionId, Function>,
    ) -> (Vec<Event>, ValueSet) {
        let mut events = Vec::new();
        let mut all_tainted = ValueSet::new(&func.dfg);

        // Traverse in Reverse Post Order, ie. top-down.
        for block_id in self.post_order.iter().rev() {
            for instruction_id in func.dfg[*block_id].instructions() {
                let instruction = &func.dfg[*instruction_id];
                let results = instruction_results(func, instruction_id);

                // Tainted values cannot be used to constrain Brillig output.
                if !results.is_empty()
                    && instruction_arguments(func, instruction)
                        .iter()
                        .any(|a| all_tainted.contains(a))
                {
                    all_tainted.extend(&results);
                }

                if is_call_to_brillig(func, all_functions, instruction_id) && !results.is_empty() {
                    // Always keep track of tainted descendants, required for correct constraint checks.
                    all_tainted.extend(&results);
                    if let Some(index) = self.tainted.index_of(instruction_id) {
                        events.push(Event::Call(index));
                    }
                } else if self.constraints.contains(instruction_id) {
                    events.push(Event::Constraint(instruction_arguments(func, instruction)));
                }
            }
        }

        (events, all_tainted)
    }

    /// A single top-down walk over the `events` attempting to constrain Brillig outputs,
    /// accumulating cleared outputs into `all_constrained`. See [`Self::constrain_tainted`].
    ///
    /// Returns `true` if at least one output was cleared during the walk.
    fn constrain_tainted_pass(
        &mut self,
        events: &[Event],
        all_tainted: &ValueSet,
        all_constrained: &mut ValueSet,
    ) -> bool {
        // Skip checks until we encounter the tainted instruction.
        let mut active = TaintedSet::default();
        // Whether any output was cleared during this walk.
        let mut progressed = false;

        for event in events {
            match event {
                Event::Call(index) => active.insert(*index),
                Event::Constraint(constrained_values) => {
                    progressed |= self.try_constrain_active(
                        constrained_values,
                        &active,
                        all_tainted,
                        all_constrained,
                    );
                }
            }
        }

        progressed
    }

    /// Try to constrain the outputs of the `active` calls which are unresolved and for which
    /// the constraint has something to do with the outputs.
    ///
    /// Returns `true` if at least one output was cleared.
    fn try_constrain_active(
        &mut self,
        constrained_values: &[ValueId],
        active: &TaintedSet,
        all_tainted: &ValueSet,
        all_constrained: &mut ValueSet,
    ) -> bool {
        let mut candidates = self.constrainable.owners_of(constrained_values);
        candidates.intersect_with(active);
        candidates.intersect_with(self.tainted.unresolved());
        if candidates.is_empty() {
            return false;
        }

        let balls =
            vecmap(constrained_values, |value| self.graph.ball(*value, self.max_ancestor_distance));

        let mut progressed = false;
        for index in candidates.iter() {
            progressed |= self.tainted.try_constrain(
                index,
                constrained_values,
                &balls,
                all_tainted,
                all_constrained,
            );
        }
        progressed
    }

    /// Every Brillig call not properly constrained should remain unresolved
    /// at this point. For each, emit a corresponding warning.
    fn into_warnings(self, function: &Function) -> Vec<SsaReport> {
        self.tainted
            .unresolved_instructions()
            .map(|brillig_call| {
                SsaReport::Bug(InternalBug::UncheckedBrilligCall {
                    call_stack: function.dfg.get_instruction_call_stack(brillig_call),
                })
            })
            .collect()
    }
}

/// Whether there is at least one instruction making a call to a Brillig function with non-empty results.
fn has_call_to_brillig(func: &Function, all_functions: &BTreeMap<FunctionId, Function>) -> bool {
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            if is_call_to_brillig(func, all_functions, instruction_id) {
                return true;
            }
        }
    }
    false
}

/// Whether the instruction is a call to a Brillig function with a non-empty results.
fn is_call_to_brillig(
    func: &Function,
    all_functions: &BTreeMap<FunctionId, Function>,
    instruction_id: &InstructionId,
) -> bool {
    let Instruction::Call { func: callee_id, .. } = func.dfg[*instruction_id] else {
        return false;
    };
    let Value::Function(callee_id) = func.dfg[callee_id] else {
        return false;
    };
    if !all_functions[&callee_id].runtime().is_brillig() {
        return false;
    }
    !func.dfg.instruction_results(*instruction_id).is_empty()
}

/// Whether an instruction puts constraints on its inputs.
fn is_constraint(func: &Function, instruction_id: &InstructionId) -> bool {
    let instruction = &func.dfg[*instruction_id];
    if matches!(
        instruction,
        Instruction::Constrain(..)
            | Instruction::ConstrainNotEqual(..)
            | Instruction::RangeCheck { .. }
    ) {
        return true;
    }
    let Instruction::Call { func: callee_id, .. } = instruction else {
        return false;
    };
    let Value::Intrinsic(intrinsic) = &func.dfg[*callee_id] else {
        return false;
    };
    matches!(intrinsic, Intrinsic::ApplyRangeConstraint | Intrinsic::AssertConstant)
}

/// Whether the instruction enables side effects with a non-constant variable.
fn is_side_effect(func: &Function, instruction: &Instruction) -> bool {
    let Instruction::EnableSideEffectsIf { condition } = instruction else {
        return false;
    };
    !is_numeric_constant(func, *condition)
}

/// Whether the instruction is a `constrain v1 == v2` with non-constant variables.
fn as_equivalence(func: &Function, instruction: &Instruction) -> Option<(ValueId, ValueId)> {
    if let Instruction::Constrain(v1, v2, _) = instruction
        && !is_numeric_constant(func, *v1)
        && !is_numeric_constant(func, *v2)
    {
        Some((*v1, *v2))
    } else {
        None
    }
}

/// Collect non-constant arguments of an instruction.
fn instruction_arguments(func: &Function, instruction: &Instruction) -> Vec<ValueId> {
    let mut arguments = Vec::new();
    // Skip the first value of calls, which is the function ID.
    let skip_first = matches!(instruction, Instruction::Call { .. });
    let mut is_first = true;
    instruction.for_each_value(|value_id| {
        if !(skip_first && is_first || is_numeric_constant(func, value_id)) {
            arguments.push(value_id);
        }
        is_first = false;
    });
    arguments
}

/// Like [`instruction_arguments`], but tailored for the parent-graph edges that
/// drive the ancestry-based check. For an [`Instruction::ArrayGet`] with a
/// *non-constant* index the result is some single element of `array`, not a
/// function of the whole array; treating `array` as a parent would propagate
/// every element's ancestry to the result, letting a constraint on the result
/// trivially "explain" outputs that are actually free for the prover when
/// `index` happens to point at a different slot.
fn parent_arguments(func: &Function, instruction: &Instruction) -> Vec<ValueId> {
    if let Instruction::ArrayGet { index, .. } = instruction
        && func.dfg.get_numeric_constant(*index).is_none()
    {
        return vec![*index];
    }
    instruction_arguments(func, instruction)
}

/// Collect non-constant results of an instruction.
fn instruction_results(func: &Function, instruction_id: &InstructionId) -> Vec<ValueId> {
    func.dfg
        .instruction_results(*instruction_id)
        .iter()
        .filter(|value| !is_numeric_constant(func, **value))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        Ssa,
        checks::check_for_missing_brillig_constraints::{
            DEFAULT_MAX_ANCESTOR_DISTANCE, DEFAULT_MAX_ARRAY_OUTPUT_LENGTH,
        },
    };
    use noirc_artifacts::ssa::SsaReport;
    use tracing_test::traced_test;

    fn check_for_missing_brillig_constraints_in_ssa(src: &str) -> Vec<SsaReport> {
        let mut ssa = Ssa::from_str(src).unwrap();
        ssa.check_for_missing_brillig_constraints(
            DEFAULT_MAX_ARRAY_OUTPUT_LENGTH,
            DEFAULT_MAX_ANCESTOR_DISTANCE,
        )
    }

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function is left unchecked with a later assert,
    /// by example of the program illustrating issue #5425 (simplified variant).
    ///
    /// The crux of this test is the load and store of values leading to the constraint.
    fn test_underconstrained_value_detector_5425() {
        /*
        unconstrained fn maximum_price(options: [u32; 2]) -> u32 {
            let mut maximum_option = options[0];
            if (options[1] > options[0]) {
                maximum_option = options[1];
            }
            maximum_option
        }

        fn main(sandwiches: pub [u32; 2], drinks: pub [u32; 2], best_value: u32) {
            let most_expensive_sandwich = maximum_price(sandwiches);
            let mut sandwich_exists = false;
            sandwich_exists |= (sandwiches[0] == most_expensive_sandwich);
            sandwich_exists |= (sandwiches[1] == most_expensive_sandwich);
            assert(sandwich_exists);

            let most_expensive_drink = maximum_price(drinks);
            assert(
                best_value
                == (most_expensive_sandwich + most_expensive_drink)
            );
        }
        */
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v4: [u32; 2], v5: [u32; 2], v6: u32):
            v8 = call f1(v4) -> u32
            v9 = allocate -> &mut u1
            store u1 0 at v9
            v10 = load v9 -> u1
            v11 = array_get v4, index u32 0 -> u32
            v12 = eq v11, v8
            v13 = or v10, v12
            store v13 at v9
            v14 = load v9 -> u1
            v15 = array_get v4, index u32 1 -> u32
            v16 = eq v15, v8
            v17 = or v14, v16
            store v17 at v9
            v18 = load v9 -> u1
            constrain v18 == u1 1    // This constrains v8
            v19 = call f1(v5) -> u32
            v20 = add v8, v19        // Combines the output of the call with v8
            constrain v6 == v20      // v6 is not connected to the inputs, so this shouldn't constrain;
            return                   // v20 is connected to the output v19, so it cannot provide the input-side constraint,
                                     // so even though the tainted v8 is constrained, it cannot be provide a constraint here.
        }

        brillig(inline) fn maximum_price f1 {
          b0(v0: [u32; 2]):
            v2 = array_get v0, index u32 0 -> u32
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a call to a Brillig function returning multiple result values
    /// is left unchecked with a later assert involving all the results
    fn test_unchecked_multiple_results_brillig() {
        // First call is constrained properly, involving both results
        // Second call is insufficiently constrained, involving only one of the results
        // The Brillig function is fake, for simplicity's sake
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2, v3 = call f1(v0) -> (u32, u32)
            v4 = mul v2, v3
            constrain v4 == v0
            v5, v6 = call f1(v0) -> (u32, u32)
            v7 = mul v5, v5
            constrain v7 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: u32):
            return u32 0, u32 0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function is called with a constant argument
    /// (should _not_ lead to a false positive failed check
    /// if all the results are constrained)
    fn test_checked_brillig_with_constant_arguments() {
        // The call is constrained properly, involving both results
        // (but the argument to the Brillig is a constant)
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v3, v4 = call f1(Field 7) -> (u32, u32)
            v5 = mul v3, v4
            constrain v5 == v0
            return
        }

        brillig(inline) fn factor f1 {
          b0(v0: Field):
            return u32 0, u32 0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig function call is constrained with a range check
    /// (should _not_ lead to a false positive failed check)
    fn test_range_checked_brillig() {
        // The call is constrained properly with a range check, involving
        // both Brillig call argument and result
        // The Brillig function is fake, for simplicity's sake

        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> u32
            v3 = add v2, v0
            range_check v3 to 32 bits
            return
        }

        brillig(inline) fn dummy f1 {
          b0(v0: u32):
            return u32 0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a Brillig nested type result is insufficiently constrained
    /// (with a field constraint missing)
    fn test_nested_type_result_brillig() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(bar: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: bar }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = foo(x);
            assert(dog.legs == 4);
            assert(dog.tag.no == x);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2, v3, v4 = call f1(v0) -> (Field, u8, Field)
            v6 = eq v2, Field 4
            constrain v2 == Field 4
            v10 = eq v4, v0
            constrain v4 == v0
            return v2, v3, v4
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test where Brillig calls' root result values are constrained against
    /// each other (covers a false negative edge case)
    /// (<https://github.com/noir-lang/noir/pull/6658#pullrequestreview-2482170066>)
    fn test_root_result_intersection_false_negative() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = call f1(v0, v1) -> Field
            v6 = eq v3, v5
            constrain v3 == v5
            v8 = add v3, v5
            return v8
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test `EnableSideEffectsIf` conditions affecting the dependency graph
    /// (SSA a bit convoluted to work around simplification breaking the flow
    /// of the parsed test code). Note that the side effect variable is a
    /// descendant of the output of the call, and the constraint is on a
    /// variable which is affected by the side effect variable.
    fn test_enable_side_effects_affecting_following_statements() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v5 = add v0, v1
            v6 = eq v3, v5
            v7 = add u1 1, u1 0
            enable_side_effects v6
            v8 = add v7, u1 1
            enable_side_effects u1 1
            constrain v8 == u1 2
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being underconstrained
    fn test_brillig_result_array_missing_element_constraint() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    /// Test call result array elements being constrained properly
    fn test_brillig_result_array_all_elements_constrained() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v16 = call f1(v0) -> [u32; 3]
            v17 = array_get v16, index u32 0 -> u32
            constrain v17 == v0
            v20 = array_get v16, index u32 1 -> u32
            constrain v20 == v0
            v19 = array_get v16, index u32 2 -> u32
            constrain v19 == v0
            return v17
        }

        brillig(inline) fn into_array f1 {
          b0(v0: u32):
            v4 = make_array [v0, v0, v0] : [u32; 3]
            return v4
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained (wrapper) Brillig calls not producing a false positive.
    ///
    /// A wrapper was considered something that passes all the outputs of
    /// one Brillig call as inputs to the next Brillig call.
    fn test_chained_brillig_calls_constrained_wrapped() {
        /*
        struct Animal {
            legs: Field,
            eyes: u8,
            tag: Tag,
        }

        struct Tag {
            no: Field,
        }

        unconstrained fn foo(x: Field) -> Animal {
            Animal {
                legs: 4,
                eyes: 2,
                tag: Tag { no: x }
            }
        }

        unconstrained fn bar(x: Animal) -> Animal {
            Animal {
                legs: x.legs,
                eyes: x.eyes,
                tag: Tag { no: x.tag.no + 1 }
            }
        }

        fn main(x: Field) -> pub Animal {
            let dog = bar(foo(x));
            assert(dog.legs == 4);
            assert(dog.eyes == 2);
            assert(dog.tag.no == x + 1);

            dog
        }
        */
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v27, v28, v29 = call f2(v0) -> (Field, u8, Field)
            v30, v31, v32 = call f1(v27, v28, v29) -> (Field, u8, Field)
            constrain v30 == Field 4
            constrain v31 == u8 2
            v35 = add v0, Field 1
            constrain v32 == v35
            return v30, v31, v32
        }

        brillig(inline) fn foo f2 {
          b0(v0: Field):
            return Field 4, u8 2, v0
        }

        brillig(inline) fn bar f1 {
          b0(v0: Field, v1: u8, v2: Field):
            v7 = add v2, Field 1
            return v0, v1, v7
        }

        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test chained Brillig calls.
    ///
    /// This is based on the diagram from the top of the module.
    fn test_chained_brillig_calls_constrained_mixed() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = mul v0, v1
            v4 = call f1(v1, v2) -> Field
            v5 = call f1(v3, v4) -> Field
            constrain v3 == v5
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Show that the output of two Brillig calls don't constrain each other.
    fn test_brillig_calls_constrained_only_against_each_other() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = call f1(v0, v1) -> Field
            v3 = call f1(v2, v2) -> Field
            constrain v2 == v3
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 2);
    }

    #[test]
    #[traced_test]
    /// Test chained Brillig calls.
    ///
    /// In this one we constrain the output of the first call against a constant,
    /// then we feed it into a second call, and constrain the second call output
    /// against its tainted input. But because the tainted input is constrained,
    /// the second call should be constrained as well.
    fn test_chained_brillig_calls_constrained_against_const_then_tainted_input() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = mul v0, v1
            v4 = call f1(v1, v2) -> Field
            constrain v4 == Field 10
            v5 = call f1(v4, v4) -> Field
            v6 = mul v4, Field 2
            constrain v5 == v6
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test for the argument descendants coming before Brillig calls themselves being
    /// registered as such
    fn test_brillig_argument_descendants_preceding_call() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = add v0, v1
            v5 = call f1(v0, v1) -> Field
            constrain v3 == v5
            return v3
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = add v0, v1
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// No-result calls (e.g. print) shouldn't trigger the check
    fn test_no_result_brillig_calls() {
        let program = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(Field 1)
            return Field 1
        }
        acir(inline) fn println f1 {
          b0(v0: Field):
            call f2(u1 1, v0)
            return
        }
        brillig(inline) fn print_unconstrained f2 {
          b0(v0: u1, v1: Field):
            return
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test for programs equivalent to the below (#10547):
    ///
    /// ```noir
    /// unconstrained fn identity(input: u64) -> u64 {
    ///     input
    /// }
    ///
    /// pub fn main(input: u32) {
    ///     let casted_input = input as u64;
    ///     let input_copy = unsafe { identity(casted_input) };
    ///     assert_eq(input_copy as Field, casted_input as Field);
    /// }
    /// ```
    fn multiple_casts_on_brillig_input_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u32):
            v1 = cast v0 as u64
            v3 = call f1(v1) -> u64
            v4 = cast v3 as Field
            v5 = cast v0 as Field
            constrain v4 == v5
            return
        }
        brillig(inline) pure fn identity f1 {
            b0(v0: u64):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn truncating_brillig_argument_does_not_result_in_warning() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
            b0(v0: Field):
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = call f1(v1) -> Field
            constrain v2 == v0
            return
        }
        brillig(inline) pure fn identity32 f1 {
            b0(v0: Field):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_independent_variable_can_indirectly_clear_results() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = call f1(v0) -> u32
            constrain v3 == v1       // This constraint does not connect the input of f1 to the output, so it doesn't clear.
            v4 = lt v1, u32 1000000  // This is a constraint against a constant, so it would clear if it was directly v3.
            constrain v4 == u1 1     // Since we asserted that v3 equals v1, this should indirectly clear v3.
            return
        }
        brillig(inline) pure fn f f1 {
          b0(v0: u32):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn constrain_on_array_element_links_to_input_array() {
        // Regression test for https://github.com/noir-lang/noir/issues/11807
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v3 = call f1(v1) -> Field
            constrain v3 == v0
            return v3
        }
        brillig(inline) pure fn helper_func f1 {
          b0(v0: [Field; 1]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn constrain_on_nested_array_element_links_to_input_array() {
        // Nested array variant: [[Field; 1]; 1] wrapping v0
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = make_array [v0] : [Field; 1]
            v2 = make_array [v1] : [[Field; 1]; 1]
            v4 = call f1(v2) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) pure fn helper_func f1 {
          b0(v0: [[Field; 1]; 1]):
            v2 = array_get v0, index u32 0 -> [Field; 1]
            v3 = array_get v2, index u32 0 -> Field
            return v3
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0, "Expected no warnings but found some.");
    }

    #[test]
    #[traced_test]
    fn array_set_with_variable_index_constrain_against_set_value() {
        // Array built from constants, then array_set with a non-constant index
        // inserts v0. Brillig result constrained against v0.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = make_array [Field 0, Field 0] : [Field; 2]
            v3 = array_set v2, index v1, value v0
            v4 = call f1(v3) -> Field
            constrain v4 == v0
            return v4
        }
        brillig(inline) pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set value should be tracked as a call argument."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_on_param_array_constrain_against_original_element() {
        // make_array [v0, v1], then array_set at non-constant index with v0.
        // Brillig result constrained against v0 (which is both in the original
        // make_array AND the array_set value).
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, v1] : [Field; 2]
            v4 = array_set v3, index v2, value v0
            v5 = call f1(v4) -> Field
            constrain v5 == v0
            return v5
        }
        brillig(inline) pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            v2 = array_get v0, index u32 0 -> Field
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: array_set on make_array with params, constrained against original element."
        );
    }

    #[test]
    #[traced_test]
    fn array_set_constrain_result_array_elements() {
        // Brillig returns an array. We array_get each element and constrain
        // against the values used in the array_set. Since the Brillig call's
        // result is an array, the checker uses array element tracking.
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u32):
            v3 = make_array [v0, Field 0] : [Field; 2]
            v4 = array_set v3, index v2, value v1
            v5 = call f1(v4) -> [Field; 2]
            v6 = array_get v5, index u32 0 -> Field
            v7 = array_get v5, index u32 1 -> Field
            constrain v6 == v0
            constrain v7 == v1
            return
        }
        brillig(inline) pure fn helper_func f1 {
          b0(v0: [Field; 2]):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(
            ssa_level_warnings.len(),
            0,
            "Expected no warnings: both array elements constrained against inputs."
        );
    }

    #[test]
    #[traced_test]
    fn outputs_do_not_trivially_connect_to_inputs() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1, v2 = call f1(v0) -> (u32, u32)
            constrain v1 == v2
            return
        }
        brillig(inline) pure fn f f1 {
          b0(v0: u32):
            return v0, v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(
            ssa_level_warnings.len(),
            1,
            "We are constraining the outputs, but they are *not* connected to the inputs"
        );
    }

    #[test]
    #[traced_test]
    fn single_call_no_constraint() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v1 = call f1() -> i64
            return v1
        }
        brillig(inline) predicate_pure fn func_1 f1 {
          b0():
            v2 = shl i64 0, i64 -877061792390071735
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 1);
    }

    #[test]
    #[traced_test]
    fn array_output_constant_constraint_on_sum() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = call f1(v0) -> [u32; 2]
            v2 = array_get v1, index u32 0 -> u32
            v3 = array_get v1, index u32 1 -> u32
            v4 = unchecked_add v2, v3
            v5 = lt v4, u32 100
            constrain v5 == u1 1
            return
        }
        brillig(inline) pure fn f f1 {
          b0(v0: u32):
            v1 = make_array [v0, v0] : [u32; 2]
            return v1
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    /// The array returned is longer than `MAX_ARRAY_OUTPUT_LENGTH` so we don't track it item-by-item,
    /// but the constraint placed on a few items should clear the whole array.
    #[test]
    #[traced_test]
    fn large_array_output_constant_constraint_on_sum() {
        let program = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = call f1(v0) -> [u32; 100]
            v2 = array_get v1, index u32 0 -> u32
            v3 = array_get v1, index u32 1 -> u32
            v4 = unchecked_add v2, v3
            v5 = lt v4, u32 100
            constrain v5 == u1 1
            return
        }
        brillig(inline) pure fn f f1 {
          b0(v0: u32):
            v1 = make_array [
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
              v0, v0, v0, v0, v0, v0, v0, v0, v0, v0,
            ] : [u32; 100]
            return v1
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    fn test_brillig_output_constrained_against_ancestor_of_input() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32):
            v1 = add v0, u32 10
            v2 = sub v1, u32 10
            v3 = call f1(v2) -> u32
            constrain v3 == v0
            return
        }

        brillig(inline) fn foo f1 {
          b0(v0: u32):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// A Brillig output sits at index 0 of a regular array, and the only
    /// constraint involves an `array_get` at a *non-constant* index. With
    /// `idx != 0` the assertion `arr[idx] == x` reduces to `x == x` and says
    /// nothing about the Brillig output, so the call is effectively
    /// unconstrained and the check must report a warning.
    fn dynamic_array_get_does_not_constrain_brillig_output() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = call f1(v0) -> u32
            v3 = make_array [v2, v0, v0, v0] : [u32; 4]
            v4 = array_get v3, index v1 -> u32
            constrain v4 == v0
            return v2
        }

        brillig(inline) fn evil f1 {
          b0(v0: u32):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(
            ssa_level_warnings.len(),
            1,
            "arr[idx] == x with dynamic idx does not constrain arr[0] = brillig(x)"
        );
    }

    #[test]
    #[traced_test]
    /// Regression test for <https://github.com/noir-lang/noir/issues/12506>:
    /// an equivalence between two Brillig outputs must clear the second call
    /// even when it appears *before* the constraint that pins the first output.
    /// The result must not depend on the source order of the two assertions.
    fn equivalence_before_pin_is_order_independent() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 3
            v2 = call f1(v1) -> Field
            v3 = call f1(v1) -> Field
            constrain v2 == v3   // (A) equivalence of the two outputs, seen first
            constrain v2 == v1   // (B) pins the first output to an input-derived value
            return
        }

        brillig(inline) fn read_imm f1 {
          b0(v0: Field):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Mirror of [`equivalence_before_pin_is_order_independent`] with the
    /// assertions in the opposite order, which already compiled cleanly before
    /// the fix. Both orders must now agree (no warning).
    fn pin_before_equivalence_is_order_independent() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, Field 3
            v2 = call f1(v1) -> Field
            v3 = call f1(v1) -> Field
            constrain v2 == v1   // (B) pins the first output to an input-derived value
            constrain v2 == v3   // (A) equivalence of the two outputs
            return
        }

        brillig(inline) fn read_imm f1 {
          b0(v0: Field):
            return v0
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Multi-output variant of [`equivalence_before_pin_is_order_independent`]:
    /// a single Brillig call returns two outputs, an equivalence between them is
    /// seen before the constraint that pins the first output. Clearing the first
    /// output leaves the call partially tainted, so the fixed-point loop must
    /// re-walk (tracking cleared outputs, not whole calls) to clear the second.
    fn multi_output_equivalence_before_pin_is_order_independent() {
        let program = r#"
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32):
        v1, v2 = call f1(v0) -> (u32, u32)
        constrain v1 == v2
        constrain v1 == v0
        return
    }
    brillig(inline) pure fn f f1 {
      b0(v0: u32):
        return v0, v0
    }
    "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a value read at a dynamic index from a fully constrained array output
    /// is the input of another call, which is constrained against it.
    fn test_input_read_at_dynamic_index_from_constrained_array() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 2], v1: u32):
            v3 = call f1(v0) -> [Field; 2]
            v5 = array_get v3, index u32 0 -> Field
            v6 = array_get v0, index u32 0 -> Field
            constrain v5 == v6
            v8 = array_get v3, index u32 1 -> Field
            v9 = array_get v0, index u32 1 -> Field
            constrain v8 == v9
            v10 = array_get v3, index v1 -> Field
            v11 = call f2(v10) -> Field
            v13 = mul v11, Field 2
            constrain v13 == v10
            return v11
        }

        brillig(inline) fn copy f1 {
          b0(v0: [Field; 2]):
            return v0
        }

        brillig(inline) fn half f2 {
          b0(v0: Field):
            v2 = div v0, Field 2
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 0);
    }

    #[test]
    #[traced_test]
    /// Test where a value read at a dynamic index from a partially constrained array output
    /// is the input of another call: the value may be the unconstrained item, so it can't
    /// be relied on.
    fn test_input_read_at_dynamic_index_from_partially_constrained_array() {
        let program = r#"
        acir(inline) fn main f0 {
          b0(v0: [Field; 2], v1: u32):
            v3 = call f1(v0) -> [Field; 2]
            v5 = array_get v3, index u32 0 -> Field
            v6 = array_get v0, index u32 0 -> Field
            constrain v5 == v6
            v10 = array_get v3, index v1 -> Field
            v11 = call f2(v10) -> Field
            v13 = mul v11, Field 2
            constrain v13 == v10
            return v11
        }

        brillig(inline) fn copy f1 {
          b0(v0: [Field; 2]):
            return v0
        }

        brillig(inline) fn half f2 {
          b0(v0: Field):
            v2 = div v0, Field 2
            return v2
        }
        "#;

        let ssa_level_warnings = check_for_missing_brillig_constraints_in_ssa(program);
        assert_eq!(ssa_level_warnings.len(), 2);
    }
}
