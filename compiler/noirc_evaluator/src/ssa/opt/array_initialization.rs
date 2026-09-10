//! Rewrites a chain of `array_set`s that rebuilds an array into a single `make_array`.
//!
//! A loop that fills an array (`for i in 0..N { a[i] = f(i) }`) unrolls into `N` sequential
//! `array_set` instructions, each producing a fresh `N`-element array value:
//!
//! ```text
//! v3 = make_array [Field 0, ...]   : [Field; N]
//! v5 = array_set v3, index u32 0, value v1
//! v9 = array_set v5, index u32 1, value v7
//! ...
//! ```
//!
//! Every later pass that reasons about array contents then walks `N` distinct array values of
//! `N` elements each, which is quadratic in `N`. Collapsing the chain to one `make_array` of the
//! final elements keeps the same semantics and leaves a single array value behind.
//!
//! A chain is only rewritten when it is safe to evaluate every write unconditionally:
//!   - it is rooted at a `make_array`, so all initial elements are known;
//!   - every intermediate array value has exactly one use, the next `array_set` in the chain, so
//!     no other instruction can observe a partially built array;
//!   - every index is an in-bounds constant, so no write can trap.
//!
//! Element types wider than one value need no special handling: an array of tuples flattens to one
//! `make_array` element per field, and `array_set` indexes those flat slots directly, so writing
//! `elements[index]` is right for `[(Field, Field); N]` just as it is for `[Field; N]`.
//!
//! # Ordering
//!
//! This must run after unrolling, which is what creates the chains, and before flattening, which
//! is what introduces `enable_side_effects`. A write under a predicate is conditional and cannot
//! be folded into an unconditional `make_array`, so the pass takes the absence of
//! `enable_side_effects` as a precondition rather than checking for it. It must also run before
//! `mutable_array_set_optimization`, whose in-place writes mutate the root array rather than
//! copying it, and before `brillig_array_get_and_set`, which shifts constant indices past the
//! in-memory array header so that an index no longer names the slot it writes.

use acvm::AcirField;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

/// Minimum number of element visits a rewrite has to save before it is worth doing.
///
/// A chain of `n` writes over a `len`-element array makes later passes walk `n` array values of
/// `len` elements; the `make_array` it collapses to is one value of `len` elements, so the rewrite
/// saves roughly `(n - 1) * len` element visits. Gating on that rather than on `n` alone is what
/// tracks the cost being avoided: 4 writes over a 5-element array is not the shape that hurts,
/// 4,096 writes over a 4,096-element array is.
const MIN_COLLAPSE_WORK: usize = 64;

impl Ssa {
    /// See the [module docs][self] for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn lower_array_initializations(self) -> Self {
        self.lower_array_initializations_saving_at_least(MIN_COLLAPSE_WORK)
    }

    /// [`Ssa::lower_array_initializations`] with an explicit threshold, so tests can exercise the
    /// rewrite on arrays small enough to read in a snapshot.
    fn lower_array_initializations_saving_at_least(mut self, min_work: usize) -> Self {
        for func in self.functions.values_mut() {
            #[cfg(debug_assertions)]
            array_initialization_pre_check(func);

            func.lower_array_initializations(min_work);

            #[cfg(debug_assertions)]
            array_initialization_post_check(func, min_work);
        }
        self
    }
}

/// Pre-check condition for [`Function::lower_array_initializations`].
///
/// Panics if the function contains an `enable_side_effects`, which would make a write conditional,
/// an `array_set` already marked `mutable`, which writes through to the root array instead of
/// copying it, or Brillig array indices that have been offset, which would stop a constant index
/// naming the slot it writes. See the [module docs][self] on ordering for why none can be present.
#[cfg(debug_assertions)]
fn array_initialization_pre_check(func: &Function) {
    super::checks::assert_no_brillig_array_offsets(func);
    super::checks::for_each_instruction(func, |instruction, _dfg| {
        super::checks::assert_not_enable_side_effects(instruction);
        super::checks::assert_not_mutable_array_set(instruction);
    });
}

/// Post-check condition for [`Function::lower_array_initializations`].
///
/// Panics if a chain worth collapsing survives, which is what running to a fixed point is for.
#[cfg(debug_assertions)]
fn array_initialization_post_check(func: &Function, min_work: usize) {
    assert!(
        collapsible_chains(func, min_work).is_empty(),
        "array initialization lowering left a collapsible chain behind"
    );
}

/// One `array_set` in a chain.
struct Link {
    instruction: InstructionId,
    block: BasicBlockId,
    array: ValueId,
    index: ValueId,
    value: ValueId,
}

/// A chain that is worth rewriting, and the array it was found to build.
struct Collapse {
    /// The chain's last write, reused in place as the `make_array`. Its result already has the
    /// array type, so every existing use of the finished array stays valid.
    tail: InstructionId,
    /// The writes leading up to it, which the rewrite makes dead.
    superseded: Vec<(BasicBlockId, InstructionId)>,
    elements: imbl::Vector<ValueId>,
    typ: Type,
}

impl Function {
    fn lower_array_initializations(&mut self, min_work: usize) {
        // Collapsing a chain can expose another: a chain rooted at an `array_set` is left alone
        // because the elements it starts from are unknown, and that root may have just become a
        // `make_array`. Every round removes at least one `array_set`, so this terminates.
        loop {
            let collapses = collapsible_chains(self, min_work);
            if collapses.is_empty() {
                return;
            }
            for collapse in collapses {
                self.apply_collapse(collapse);
            }
        }
    }

    /// Rewrites the chain's last write into the `make_array` it builds and drops the rest.
    fn apply_collapse(&mut self, collapse: Collapse) {
        let Collapse { tail, superseded, elements, typ } = collapse;
        self.dfg[tail] = Instruction::MakeArray { elements, typ };

        let mut dead: HashMap<BasicBlockId, HashSet<InstructionId>> = HashMap::default();
        for (block, instruction) in superseded {
            dead.entry(block).or_default().insert(instruction);
        }
        for (block, dead) in dead {
            self.dfg[block].instructions_mut().retain(|instruction| !dead.contains(instruction));
        }
    }
}

/// Finds every chain in `func` worth collapsing. Chains are disjoint, because an intermediate
/// belongs to the one chain that consumes it, so the results can all be applied.
fn collapsible_chains(func: &Function, min_work: usize) -> Vec<Collapse> {
    let uses = count_uses(func);
    let links = collect_links(func);

    // An array value consumed only by the next write is interior to a chain, so any `array_set`
    // result that is not one of those ends a chain.
    let interior: HashSet<ValueId> = links
        .values()
        .filter(|link| links.contains_key(&link.array) && uses[&link.array] == 1)
        .map(|link| link.array)
        .collect();

    let mut tails: Vec<ValueId> =
        links.keys().copied().filter(|result| !interior.contains(result)).collect();
    // `links` is a hash map, so sort for a deterministic rewrite order.
    tails.sort_unstable();

    tails
        .into_iter()
        .filter_map(|tail| {
            let (chain, root) = chain_ending_at(tail, &links, &uses);
            let (elements, typ) = collapsed_array(func, &chain, root, min_work)?;
            Some(Collapse {
                tail: chain[0].instruction,
                superseded: chain[1..].iter().map(|link| (link.block, link.instruction)).collect(),
                elements,
                typ,
            })
        })
        .collect()
}

/// Counts how many times each value is used, so a chain can tell an array it alone consumes from
/// one something else can still read.
fn count_uses(func: &Function) -> HashMap<ValueId, u32> {
    let mut uses: HashMap<ValueId, u32> = HashMap::default();
    for block in func.reachable_blocks() {
        for instruction in func.dfg[block].instructions() {
            func.dfg[*instruction].for_each_value(|value| {
                *uses.entry(value).or_default() += 1;
            });
        }
        if let Some(terminator) = func.dfg[block].terminator() {
            terminator.for_each_value(|value| {
                *uses.entry(value).or_default() += 1;
            });
        }
    }
    uses
}

/// Indexes every non-mutable `array_set` in `func` by the array value it produces.
fn collect_links(func: &Function) -> HashMap<ValueId, Link> {
    let mut links = HashMap::default();
    for block in func.reachable_blocks() {
        for instruction in func.dfg[block].instructions() {
            if let Instruction::ArraySet { array, index, value, mutable: false } =
                func.dfg[*instruction]
            {
                let result = func.dfg.instruction_results(*instruction)[0];
                links
                    .insert(result, Link { instruction: *instruction, block, array, index, value });
            }
        }
    }
    links
}

/// Walks back from `tail` along the `array` operand for as long as each array value is used only
/// by the next write. Returns the chain, last write first, and the array it starts from.
fn chain_ending_at<'links>(
    tail: ValueId,
    links: &'links HashMap<ValueId, Link>,
    uses: &HashMap<ValueId, u32>,
) -> (Vec<&'links Link>, ValueId) {
    let mut chain = Vec::new();
    let mut current = tail;
    loop {
        let Some(link) = links.get(&current) else { return (chain, current) };
        chain.push(link);
        if uses[&link.array] != 1 {
            return (chain, link.array);
        }
        current = link.array;
    }
}

/// Applies `chain` to the elements of `root`, giving the array the chain builds, or `None` if the
/// chain cannot be collapsed or is not worth collapsing.
fn collapsed_array(
    func: &Function,
    chain: &[&Link],
    root: ValueId,
    min_work: usize,
) -> Option<(imbl::Vector<ValueId>, Type)> {
    // A single write cannot be shortened, and the saving is only known once the array's length is.
    if chain.len() < 2 {
        return None;
    }
    let (mut elements, typ) = func.dfg.get_array_constant(root)?;
    if (chain.len() - 1) * elements.len() < min_work {
        return None;
    }

    // `chain` runs last-write-first; the writes take effect starting from the root.
    for link in chain.iter().rev() {
        let index = func.dfg.get_numeric_constant(link.index)?.try_to_u32()?;
        // Out of bounds: the write would trap, which `make_array` would not reproduce.
        let slot = elements.get_mut(index as usize)?;
        *slot = link.value;
    }
    Some((elements, typ))
}
#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change};

    use super::Ssa;

    /// A chain that overwrites every element collapses into the array it builds.
    #[test]
    fn collapses_full_initialization_chain() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 3, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v6 = make_array [v0, v1, v2, v3] : [Field; 4]
            return v6
        }
        ");
    }

    /// Elements the chain never writes keep the value the root `make_array` gave them.
    #[test]
    fn keeps_elements_the_chain_does_not_write() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 7, Field 7, Field 7, Field 7, Field 7] : [Field; 5]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 3, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 7, Field 7, Field 7, Field 7, Field 7] : [Field; 5]
            v6 = make_array [v0, v1, v2, v3, Field 7] : [Field; 5]
            return v6
        }
        ");
    }

    /// A later write to an index an earlier one already set wins.
    #[test]
    fn later_write_to_the_same_index_wins() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 0, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v6 = make_array [v3, v1, v2] : [Field; 3]
            return v6
        }
        ");
    }

    /// A chain saving fewer element visits than the threshold is left alone: here 2 writes over a
    /// 3-element array save 6, under the 8 the test asks for.
    #[test]
    fn does_not_collapse_chain_below_threshold() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v3 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v4 = array_set v3, index u32 0, value v0
            v5 = array_set v4, index u32 1, value v1
            v6 = array_set v5, index u32 2, value v2
            return v6
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.lower_array_initializations_saving_at_least(8));
    }

    /// If anything reads an intermediate array the chain is abandoned: the walk stops at that array,
    /// which is an `array_set` rather than a `make_array`, so the untouched elements are unknown.
    #[test]
    fn does_not_collapse_a_chain_with_an_observed_intermediate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 3, value v3
            v9 = array_get v6, index u32 0 -> Field
            return v8, v9
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.lower_array_initializations_saving_at_least(8));
    }

    /// A non-constant index could hit any element, so the resulting array is not known.
    #[test]
    fn does_not_collapse_non_constant_index() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: u32):
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v6 = array_set v5, index u32 0, value v0
            v7 = array_set v6, index u32 1, value v1
            v8 = array_set v7, index v4, value v2
            v9 = array_set v8, index u32 3, value v3
            return v9
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.lower_array_initializations_saving_at_least(8));
    }

    /// Writes under a predicate are conditional, so the pass requires flattening not to have run
    /// yet rather than checking for them; the pre-check is what holds that ordering in place.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic = "enable_side_effects instruction found"]
    fn pre_check_rejects_a_predicate() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: u1):
            enable_side_effects v4
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v6 = array_set v5, index u32 0, value v0
            v7 = array_set v6, index u32 1, value v1
            v8 = array_set v7, index u32 2, value v2
            v9 = array_set v8, index u32 3, value v3
            return v9
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.lower_array_initializations_saving_at_least(8);
    }

    /// After `brillig_array_get_and_set` a constant index is shifted past the in-memory array
    /// header, so it no longer names the slot it writes and the elements cannot be folded by index.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic = "Brillig array indices have already been offset"]
    fn pre_check_rejects_offset_brillig_indices() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = array_set v4, index u32 1 minus 1, value v0
            v6 = array_set v5, index u32 2 minus 1, value v1
            v7 = array_set v6, index u32 3 minus 1, value v2
            v8 = array_set v7, index u32 4 minus 1, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.lower_array_initializations_saving_at_least(8);
    }

    /// A `mutable` write goes through to the root array instead of copying it, so the chain no
    /// longer describes a value being built up.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic = "Mutable array set instruction found"]
    fn pre_check_rejects_a_mutable_write() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = array_set mut v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 3, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.lower_array_initializations_saving_at_least(8);
    }

    /// An array of tuples flattens to two `make_array` elements per index, and `array_set` writes
    /// those flat slots one at a time, so a chain over one covers two slots per source-level index.
    #[test]
    fn collapses_chain_over_a_composite_element_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = make_array [Field 0, Field 0, Field 0, Field 0] : [(Field, Field); 2]
            v3 = array_set v2, index u32 0, value v0
            v4 = array_set v3, index u32 1, value v1
            v5 = array_set v4, index u32 2, value v1
            v6 = array_set v5, index u32 3, value v0
            return v6
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = make_array [Field 0, Field 0, Field 0, Field 0] : [(Field, Field); 2]
            v4 = make_array [v0, v1, v1, v0] : [(Field, Field); 2]
            return v4
        }
        ");
    }

    /// A chain that writes only one field of each tuple leaves the other at its initial value.
    #[test]
    fn collapses_partial_chain_over_a_composite_element_type() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 9, Field 9, Field 9, Field 9, Field 9, Field 9, Field 9, Field 9] : [(Field, Field); 4]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 2, value v1
            v7 = array_set v6, index u32 4, value v2
            v8 = array_set v7, index u32 6, value v3
            return v8
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 9, Field 9, Field 9, Field 9, Field 9, Field 9, Field 9, Field 9] : [(Field, Field); 4]
            v6 = make_array [v0, Field 9, v1, Field 9, v2, Field 9, v3, Field 9] : [(Field, Field); 4]
            return v6
        }
        ");
    }

    /// The production threshold leaves the small chains alone that ordinary code is full of - a
    /// Poseidon sponge state update looks exactly like a short initialisation chain.
    #[test]
    fn production_threshold_ignores_a_small_chain() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v4 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v5 = array_set v4, index u32 0, value v0
            v6 = array_set v5, index u32 1, value v1
            v7 = array_set v6, index u32 2, value v2
            v8 = array_set v7, index u32 3, value v3
            return v8
        }
        ";
        assert_ssa_does_not_change(src, Ssa::lower_array_initializations);
    }

    /// Without a `make_array` root the untouched elements are unknown.
    #[test]
    fn does_not_collapse_chain_rooted_at_an_opaque_array() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: Field, v2: Field, v3: Field, v4: Field):
            v5 = array_set v0, index u32 0, value v1
            v6 = array_set v5, index u32 1, value v2
            v7 = array_set v6, index u32 2, value v3
            v8 = array_set v7, index u32 3, value v4
            return v8
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.lower_array_initializations_saving_at_least(8));
    }
}
