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

/// Chains shorter than this are left alone: the win comes from removing many array values, and
/// rewriting a couple of writes only churns the IR.
const MIN_CHAIN_LENGTH: usize = 4;

impl Ssa {
    /// See the [module docs][self] for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn lower_array_initializations(mut self) -> Self {
        for func in self.functions.values_mut() {
            func.lower_array_initializations();
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
    fn lower_array_initializations(&mut self) {
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
            if chain.len() < MIN_CHAIN_LENGTH {
                continue;
            }

            let Some((mut elements, typ)) = self.dfg.get_array_constant(root) else {
                continue;
            };
            // Composite element types flatten several values per index, so an index is not a
            // direct offset into `elements`. Only handle the flat case.
            if typ.element_size().0 != 1 {
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
