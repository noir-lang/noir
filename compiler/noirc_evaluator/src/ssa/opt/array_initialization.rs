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
//!   - every index is an in-bounds constant, so no write can trap;
//!   - no `array_set` in the chain is marked `mutable`, which would make the write an in-place
//!     mutation of the root rather than a copy;
//!   - the function contains no `enable_side_effects`, which would put writes under a predicate.
//!
//! The last condition means this must run before flattening, which is also where it is most
//! useful: it removes the chain before the passes that would pay the quadratic cost on it.
//!
//! Element types wider than one value need no special handling: an array of tuples flattens to one
//! `make_array` element per field, and `array_set` indexes those flat slots directly, so writing
//! `elements[index]` is right for `[(Field, Field); N]` just as it is for `[Field; N]`.

use acvm::AcirField;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

/// Minimum number of element visits a rewrite has to save before it is worth doing.
///
/// A chain of `n` writes over an `len`-element array makes later passes walk `n` array values of
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
            func.lower_array_initializations(min_work);
        }
        self
    }
}

/// One `array_set` in a chain.
struct Link {
    instruction: InstructionId,
    block: BasicBlockId,
    array: ValueId,
    index: ValueId,
    value: ValueId,
}

impl Function {
    fn lower_array_initializations(&mut self, min_work: usize) {
        let blocks = self.reachable_blocks();

        // Writes under a predicate are conditional, so a chain of them cannot be collapsed into an
        // unconditional `make_array`. `enable_side_effects` only appears after flattening.
        for block in &blocks {
            for instruction in self.dfg[*block].instructions() {
                if matches!(self.dfg[*instruction], Instruction::EnableSideEffectsIf { .. }) {
                    return;
                }
            }
        }

        let mut uses: HashMap<ValueId, u32> = HashMap::default();
        let mut links: HashMap<ValueId, Link> = HashMap::default();

        for block in &blocks {
            for instruction in self.dfg[*block].instructions() {
                self.dfg[*instruction].for_each_value(|value| {
                    *uses.entry(value).or_default() += 1;
                });

                if let Instruction::ArraySet { array, index, value, mutable: false } =
                    self.dfg[*instruction]
                {
                    let results = self.dfg.instruction_results(*instruction);
                    links.insert(
                        results[0],
                        Link { instruction: *instruction, block: *block, array, index, value },
                    );
                }
            }
            if let Some(terminator) = self.dfg[*block].terminator() {
                terminator.for_each_value(|value| {
                    *uses.entry(value).or_default() += 1;
                });
            }
        }

        // A chain link is an array value consumed only by the next `array_set`, so any `array_set`
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

        let mut removed: Vec<(BasicBlockId, InstructionId)> = Vec::new();

        for tail in tails {
            // Walk back along the `array` operand for as long as each array value is used only by
            // the next write.
            let mut chain: Vec<&Link> = Vec::new();
            let mut current = tail;
            let root = loop {
                let Some(link) = links.get(&current) else { break current };
                chain.push(link);
                if uses[&link.array] != 1 {
                    break link.array;
                }
                current = link.array;
            };
            // A single write cannot be shortened, and the saving is only known once the array's
            // length is, so the work check comes after the root is resolved.
            if chain.len() < 2 {
                continue;
            }

            let Some((mut elements, typ)) = self.dfg.get_array_constant(root) else {
                continue;
            };
            if (chain.len() - 1) * elements.len() < min_work {
                continue;
            }

            // `chain` runs tail-first; the writes take effect root-first.
            let mut applied = true;
            for link in chain.iter().rev() {
                let Some(index) =
                    self.dfg.get_numeric_constant(link.index).and_then(|index| index.try_to_u32())
                else {
                    applied = false;
                    break;
                };
                let Some(slot) = elements.get_mut(index as usize) else {
                    // Out of bounds: the write would trap, which `make_array` would not reproduce.
                    applied = false;
                    break;
                };
                *slot = link.value;
            }
            if !applied {
                continue;
            }

            // Rewrite the last write in place. Its result already has the array type, which is
            // what `make_array` produces, so every existing use stays valid.
            let tail_link = chain[0];
            self.dfg[tail_link.instruction] = Instruction::MakeArray { elements, typ };
            for link in &chain[1..] {
                removed.push((link.block, link.instruction));
            }
        }

        if removed.is_empty() {
            return;
        }
        let mut dead: HashMap<BasicBlockId, HashSet<InstructionId>> = HashMap::default();
        for (block, instruction) in removed {
            dead.entry(block).or_default().insert(instruction);
        }
        for (block, dead) in dead {
            self.dfg[block].instructions_mut().retain(|instruction| !dead.contains(instruction));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field):
            v4 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v6 = array_set v4, index u32 0, value v0
            v8 = array_set v6, index u32 1, value v1
            v10 = array_set v8, index u32 2, value v2
            return v10
        }
        ");
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v7 = array_set v5, index u32 0, value v0
            v9 = array_set v7, index u32 1, value v1
            v11 = array_set v9, index u32 2, value v2
            v13 = array_set v11, index u32 3, value v3
            v14 = array_get v9, index u32 0 -> Field
            return v13, v14
        }
        ");
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: u32):
            v6 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v8 = array_set v6, index u32 0, value v0
            v10 = array_set v8, index u32 1, value v1
            v11 = array_set v10, index v4, value v2
            v13 = array_set v11, index u32 3, value v3
            return v13
        }
        ");
    }

    /// Writes under a predicate are conditional and cannot be folded into an unconditional array.
    #[test]
    fn does_not_collapse_under_a_predicate() {
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
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: u1):
            enable_side_effects v4
            v6 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v8 = array_set v6, index u32 0, value v0
            v10 = array_set v8, index u32 1, value v1
            v12 = array_set v10, index u32 2, value v2
            v14 = array_set v12, index u32 3, value v3
            return v14
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations();
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: Field, v3: Field):
            v5 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v7 = array_set v5, index u32 0, value v0
            v9 = array_set v7, index u32 1, value v1
            v11 = array_set v9, index u32 2, value v2
            v13 = array_set v11, index u32 3, value v3
            return v13
        }
        ");
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_array_initializations_saving_at_least(8);
        assert_ssa_snapshot!(ssa, @"
        acir(inline) fn main f0 {
          b0(v0: [Field; 4], v1: Field, v2: Field, v3: Field, v4: Field):
            v6 = array_set v0, index u32 0, value v1
            v8 = array_set v6, index u32 1, value v2
            v10 = array_set v8, index u32 2, value v3
            v12 = array_set v10, index u32 3, value v4
            return v12
        }
        ");
    }
}
