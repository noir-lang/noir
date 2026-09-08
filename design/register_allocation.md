# Brillig register allocation

Brillig has no hardware register file; "registers" are the low, densely-packed slots of a
function's stack frame, addressed `Relative` to the stack pointer. Allocation decides which SSA
value lives in which slot, and what to do when a frame runs out of slots. This note records why
the current design looks the way it does, its limitations, and a proposed evolution toward a
linear-scan allocator (noir-lang/noir#11638).

## Contents

- [Cost model: this is not the textbook problem](#cost-model-this-is-not-the-textbook-problem)
- [Current design](#current-design)
  - [Allocation (no spilling)](#allocation-no-spilling)
  - [Running out of slots: LRU spilling](#running-out-of-slots-lru-spilling)
  - [Two liveness structures, distinct roles](#two-liveness-structures-distinct-roles)
- [Why the current design costs opcodes](#why-the-current-design-costs-opcodes)
- [Holes: what they are, and what they are not](#holes-what-they-are-and-what-they-are-not)
  - [Hole examples](#hole-examples)
  - [Representation](#representation)
  - [Holes are required, not optional packing](#holes-are-required-not-optional-packing)
- [Proposed evolution](#proposed-evolution)
  - [Phase 0: restore general parallel moves](#phase-0-restore-general-parallel-moves)
  - [Phase 0.5: extract a pluggable allocator seam](#phase-05-extract-a-pluggable-allocator-seam)
  - [Phase 1a: swap the victim heuristic (LRU to next-use)](#phase-1a-swap-the-victim-heuristic-lru-to-next-use)
  - [Phase 1: interval-set liveness, linear-scan assignment, resolution](#phase-1-interval-set-liveness-linear-scan-assignment-resolution)
- [Cross-cutting concerns](#cross-cutting-concerns)
  - [Coalescing](#coalescing)
  - [Temporaries and procedures](#temporaries-and-procedures)
  - [A single instruction must fit in registers at once](#a-single-instruction-must-fit-in-registers-at-once)
  - [Streaming operands: MakeArray](#streaming-operands-makearray)
- [Relationship to noir-lang/noir#11638](#relationship-to-noir-langnoir11638)
- [Validation](#validation)
  - [Not in scope: in-place spill operands](#not-in-scope-in-place-spill-operands)
- [Delivery checklist](#delivery-checklist)

## Cost model: this is not the textbook problem

The register-allocation literature optimizes for **latency**: a register access is cheap, a
spill/reload is an expensive memory round-trip, and a `mov` is nearly free. Under that model,
splitting a value's live range to reuse a register (emitting a `mov`) almost always beats
spilling.

Brillig's cost is **static opcode count** (which maps to proving cost), and for hot code the
**loop-weighted dynamic opcode count**. A `mov`, a `Load`, and an `add` all cost roughly one
opcode. So the literature's heuristics are tuned for an objective that is not ours and cannot be
ported blindly: any trade that spends extra moves to avoid a spill must be re-justified when a move
and a reload cost the same, and any victim heuristic that ignores loop depth misprices a spill
inside a hot loop. Every decision below is measured in emitted (and executed) opcodes, not memory
latency.

## Current design

### Allocation (no spilling)

Liveness is precomputed per block by `VariableLiveness` (`live_in`/`live_out`/`last_uses`).
Codegen then walks each block greedily:

- Each block gets a **fresh allocator**, re-seeded from the block's live-in set via
  `set_allocated_registers` (`BrilligBlock::compile_block`).
- `define_variable` picks a free slot; the RAII `Allocated` wrapper is immediately `detach`ed so
  its slot is *not* auto-freed, and `remove_variable` frees it manually at the value's last use —
  because a value defined in one block is deallocated in a *different* block's allocator.
- Block parameters are handled by coalescing (reuse the argument's slot) when possible, with
  simultaneous assignment at jmp handled by a parallel move (see Phase 0).

This is simple and fast, and a value actually *keeps* its slot across blocks: `ssa_value_allocations`
caches each value's register, and `set_allocated_registers` re-seeds every block with the live-in
values pinned to their cached registers — only the free-list and newly-defined values are re-derived
per block (a slot is reusable in a block where its owner is not live). **Stable identity breaks only
under spilling**, where a reload picks any free register (see below); that is what forces cross-block
dumping, not the per-block reset itself. So the non-spilling path already has stable homes — the
allocator's win is concentrated in the spilling regime.

### Running out of slots: LRU spilling

Frames are bounded (`max_stack_frame_size`). Pre-spill, overflowing a frame was a hard compile
error — `Stack::allocate_register` asserts "Stack frame too deep". Spilling (noir-lang/noir#11556)
replaced that abort with a heap-backed escape hatch.

- **Activation** is a whole-function decision:
  `max_live_count + SPILL_MARGIN >= max_stack_frame_size` constructs a `SpillManager`; otherwise
  coalescing runs (the two are mutually exclusive).
- **Victim selection** is an **LRU** over register-resident values. Before defining a value or
  making room for inlined scratch, `ensure_register_capacity` evicts the least-recently-used value
  to a heap slot. When a spilled value is next used it is reloaded into *whatever register is
  free* — not necessarily the one it left.
- **Cross-block values are dumped.** Because a reloaded value can land in a different slot, a
  successor block cannot predict where a predecessor left a value. So every value live across a
  block boundary is forced to a **permanent** heap spill (`spill_non_param_live_ins` + params
  written by each terminator); the heap slot becomes the one location all paths agree on. This is a
  deliberate simplification standing in for phi resolution. Permanent slots are never freed (the
  leak in noir-lang/noir#11695); transient slots *are* reused via a free list (`free_spill_slots`,
  `max_spill_offset`).

`live_intervals.rs` already implements Wimmer & Franz's `BUILDINTERVALS`, but collapses each value
to a single `[def, last_use]` interval and is currently `#[cfg(test)]`-only. It exists as the
foundation for the allocator proposed here, not as a live analysis.

### Two liveness structures, distinct roles

The allocator relies on two analyses that look redundant but are not:

- **`VariableLiveness` is the CFG-level truth.** Per-block `live_in`/`live_out`/`last_uses` from a
  loop-aware fixpoint. It answers "what is live at this block boundary," and is the source of both
  holes and resolution (which values cross each edge). Codegen uses it directly to free slots at
  last use and to drive edge resolution.
- **`LiveIntervals` is the linear projection the scan consumes.** It imposes a total program-point
  order and derives each value's range over it. Linear scan needs a linear sweep order the CFG does
  not provide; this is the view built *from* `VariableLiveness` for the allocation pass, and is
  derived/disposable.

Rule of thumb: "what is live across this edge" is `VariableLiveness`; "scan order and interference"
is `LiveIntervals`.

## Why the current design costs opcodes

Two structural costs, both traceable to "no stable home for a value":

1. **Cross-block dumping.** Every cross-block-live value is stored to and reloaded from the heap at
   boundaries, whether or not registers were actually scarce on that path. This is loop-amplified:
   a dump on a loop back-edge re-executes every iteration.
2. **Reload churn.** A value is reloaded to an arbitrary free slot, so its location wanders even
   within a function; this is what *forces* cost (1).

Plus two quality gaps:

3. **Myopic eviction.** LRU is a backward-looking proxy; it can evict a value used immediately next
   while keeping one used far later. Its blind spot is exactly "not recently used but about to be
   used."
4. **Over-approximated liveness (in a naive interval model).** A single `[def, last_use]` interval
   treats a value as live across control-flow regions where it is actually dead, inflating measured
   register pressure. (Note: today's `max_live_count` does *not* have this flaw — see below — but a
   naive interval allocator would reintroduce it.)

## Holes: what they are, and what they are not

A **hole** is a region where a value is dead *between* two live regions of the same value. Getting
this right is load-bearing for the whole design, and it is easy to get wrong.

**Definition (from the literature).** In interval-based linear scan a value's *lifetime interval*
is a list of disjoint live ranges over the linearized program-point order; the gaps between
consecutive ranges are its **lifetime holes** — points inside the value's overall span where it is
not live, so its register may be handed to another interval there. Poletto & Sarkar's original
linear scan uses a single conservative interval per value — one that, in their words, ignores the
subranges where the value is not live; tracking holes explicitly comes from the earlier binpacking
work they contrast against, and is carried into linear-scan-on-SSA by Wimmer et al. Sources to look
them up:

- Poletto & Sarkar, *Linear Scan Register Allocation*, 1999 — single-interval baseline.
- Wimmer & Mössenböck, *Optimized Interval Splitting in a Linear Scan Register Allocator*, 2005 — intervals as range lists with lifetime holes.
- Wimmer & Franz, *Linear Scan Register Allocation on SSA Form*, 2010 (DOI 10.1145/1772954.1772979) — the SSA-form variant `live_intervals.rs` is modeled on.
- Accessible walkthrough: <https://bernsteinbear.com/blog/linear-scan-lifetime-holes/>.

- **Unused ≠ hole.** A value is *live* (must be preserved) at every point on a path from its
  definition to a future use, even where it is never read. Consider a value defined in `b0` and
  used only in `b5`, with the diamond `b0→{b1,b2}`, `b2→{b3,b4}`, `{b1,b3,b4}→b5`. Every block lies
  on some `b0→b5` path, so the value is live throughout — there is **no hole anywhere**, even
  though it is unused in `b1..b4`. Collapsing those blocks to a straight chain makes this obvious: a
  chain has no branching and no holes for an end-used value.
- **A hole requires the region to be off *every* def→use path.** The classic case: a value used
  only on one branch of a diamond is dead on the other branch, so that branch is a hole. The test
  is one question: *does some forward path from this block reach a use?* Some path → live (no hole,
  the value must be preserved because that path might execute); no path → hole. It is binary — some
  versus none — with no "all paths" tier.
- **Holes are block-granular, by necessity.** A Brillig basic block is straight-line and values are
  single-def, so within one block a value's live range is always contiguous — there is no room for
  a hole inside a block. A hole is always a whole block (or run of blocks).
- **A hole is register sharing, not spilling — zero memory traffic.** By the definition of
  liveness, "dead in block `H`" means no path forward from `H` reaches a use, so from inside a hole
  the value is dead on *every* continuation — it is never read again on that path. And its live copy
  is reached by the *bypass* path that skips the hole, which therefore never clobbered the slot. So
  another value may take the slot during the hole with **no `Store` and no `Load`**: the two values
  simply get the same register because their live ranges are mutually exclusive at runtime. Holes
  are resolved at the *assignment* level (they lower the interference graph); the spill machinery
  that emits `Store`/`Load` is only for genuine pressure — a still-live value evicted to make room,
  which is the only case that must be reloaded.

**Holes vs pressure-spills.** Two different reasons a value leaves a register:

- *Hole*: the value is genuinely dead in a region; its slot is free there for others. No memory
  traffic.
- *Pressure spill*: the value is live (must be preserved) but evicted to memory because the block
  ran out of registers. This *does* cost a store and a later reload. The `b0..b5` example above is
  pure pressure spilling — the value is live everywhere, so any spill there is pressure, not a hole.

**Holes vs interval splitting.** Do not confuse a hole with *interval splitting*, though both leave
a gap in register-residency. A hole is *given* — a region where the value is dead, free to exploit
(register sharing, no memory). Splitting is *chosen* — the allocator deliberately cuts a *live*
interval at a point under pressure so one sub-range is in a register and another in a spill slot,
inserting a store/reload at the cut; it is a form of pressure spilling and costs opcodes. A hole is
where the value doesn't exist; a split point is where it is live but evicted anyway. Poletto has
neither (single interval, spill whole); Wimmer 2010 has both.

### Hole examples

**Diamond — a real hole, and two intervals separated by it.** `v` defined in `b0`, used only in
`b2`; `w` defined and used only in `b1`:

```
    b0        v = ...                 ; v defined
   /  \       edges: b0->b1, b0->b2, b1->b3, b2->b3
  b1   b2     b1: w = ...; use(w)     ; w lives only in b1
   \  /       b2: use(v)              ; v used only in b2
    b3        neither v nor w used
```

`v` is live in `{b0, b2}` and dead in `b1` and `b3`; in the linear order `b0,b1,b2,b3` its envelope
`[def@b0, last_use@b2]` spans `b1`, so **`b1` is a hole**. `w` lives only in `b1` — exactly inside
`v`'s hole. Over the linear order:

```
block:   b0        b1        b2
v:       live      hole      live      # interval [b0..b2], dead in b1
w:        –        live       –        # interval b1 only
```

`v` and `w` are never live at the same point, so **they do not interfere and can be assigned the
same register** — it holds `v` in `b0`/`b2` and `w` in `b1`. *That* is "two intervals separated by a
hole." It is safe with no save/restore because `b1` and `b2` are mutually exclusive at runtime (`b1`
executing means the `b2` use of `v` is never reached, and vice versa). `b3` is *after* `v`'s last
use, so it is not a hole — the range simply ends.

**Loops mostly suppress holes.** The tempting-but-wrong case: `v` defined before a loop, used after,
unused in the body:

```
  b0    v = ...
  |     edges: b0->b1, b1->b2, b2->b1 (back-edge), b1->b3 (exit)
  b1 <--+   loop header
  |     |
  b2 ---+   body — does NOT use v
  |
  b3    use(v)     (after the loop)
```

This is **not** a hole: `v` must survive every iteration to reach the use in `b3`, so it is
live-through `b1` and `b2` (a pressure spill here would be exactly that — a spill, not a hole).
More generally, if `v` were used *anywhere* in the body, the back-edge makes that use reachable from
every block in the loop, so `v` is live across the whole loop body — still no hole. This back-edge
closure is why `VariableLiveness` is loop-aware, and why genuine holes almost never occur *inside* a
loop.

**A loop can only sit *inside* a hole.** The one loop-involving hole is the diamond case with a loop
on the dead arm: `v` defined in `b0`, a loop on the left branch, `v` used only on the right branch.
The entire left branch — loop and all — is off every path to the use, so it is one big hole and the
slot is reusable throughout it. The loop is incidental; it does not *create* the hole, it merely
lives in one.

### Representation

Keep `LiveInterval { def, last_use }` as the envelope (the scan-order key and interference's cheap
first test), and attach holes only as an exception — a small list of *blocks*, not arbitrary
program-point ranges. Most values have no holes, so this avoids a per-value `Vec` and keeps the
common-case interference check unchanged; two values interfere iff their envelopes overlap and the
overlap isn't entirely swallowed by a hole of one side. Holes are derivable from
`VariableLiveness` (a hole = a block where the value is neither live-in, live-out, nor used).

The envelope is a projection of the CFG live-set onto the chosen linear order, so a *hole* is a
property of that order — a dead block that happens to sort between two live ones — while liveness
itself is intrinsic. A different block ordering relocates holes (a dead block may sort past
`last_use`, becoming range-end rather than a gap) without changing which blocks are live. This is
Wimmer & Franz's own framing — *"Lifetime holes occur because the control flow graph is reduced to
a list of blocks before register allocation"* — and Poletto & Sarkar likewise call the single
interval *"a conservative approximation … there may be subranges of [i, j] in which v is not live,
but they are ignored,"* its accuracy depending on the instruction-numbering order.

### Holes are required, not optional packing

`VariableLiveness::max_live_count` — today's spill trigger — is **not** a single-interval envelope
count. It walks each block instruction-by-instruction, subtracting `last_uses`, so it is already
the *accurate* peak of simultaneously-live values (a value in a hole is not counted there). A naive
single-interval `LiveIntervals` over-approximates that peak, so a linear scan built on single
intervals would spill in cases that currently fit — a **regression**. The minimum achievable slot
count *is* `max_live_count`, and holes are the mechanism by which an interval allocator reaches it.
So holes are a **prerequisite** for a non-regressing allocator, not a "later" refinement: a rollout
that ships single-interval linear scan first would regress at that step. The floor is *block-granular*
holes — the precision the current greedy+LRU already has implicitly via its per-block reseed.

So distinguish two quantities:

- **Spill incidence** (whether a program spills at all) is governed by peak pressure. Holes are
  needed only so the interval allocator *matches* the current trigger and does not regress.
- **Spill volume** (how many spill/reload opcodes are emitted and executed) is where the allocator
  wins, via fixed homes + resolution — and it is largely independent of holes.

Volume, not incidence, is the payoff.

## Proposed evolution

The goal is a **global** slot assignment: every value (or split sub-range) has one agreed home for
its lifetime. That single change unlocks a cheap **resolution** phase — at a control-flow edge,
emit only the moves where the two sides' assignments differ, instead of dumping the whole
live-across set. The phases below are ordered to land value early and de-risk incrementally.

### Phase 0: restore general parallel moves

**Landed in noir-lang/noir#13307.** The restriction was lifted from
`codegen_mov_registers_to_registers` itself (so one routine serves both return-value copies and
general moves, and takes an optional `condition`), and `jmp_setup` now calls it; a block that
rotates its parameters drops from one temporary per parameter to a single cycle-breaking temporary.
The problem statement below is kept for context.

`codegen_mov_registers_to_registers` was a general any-source→any-destination parallel-move solver
(noir-lang/noir#6089): a movement graph with cycle detection and a scratch register to break
cycles. noir-lang/noir#10305 specialized it to **consecutive destination slots** for the
return-value copy path (`assert to_index(dst[i]) == i`), keeping the cycle machinery but dropping
arbitrary destinations.

Resolution across a CFG edge is exactly the out-of-SSA parallel-copy problem (Boissinot et al.,
"Revisiting Out-of-SSA Translation"): simultaneous `src → dst` with **arbitrary** destinations,
cycles broken by a scratch slot. That is the capability #10305 removed.

The constraint bites *today*: block-parameter passing at a jmp cannot call the consecutive-only
function, so `jmp_setup` carries its own inline mover that **saves every source-that-is-also-a-
destination into a fresh temp** before moving. That is correct but strictly worse than the general
algorithm — it spends a temp (and an extra `mov`) per conflicting source, where the general solver
orders acyclic chains with *zero* temps and uses one scratch *per cycle*:

| jmp pattern    | inline mover        | general solver     |
|----------------|---------------------|--------------------|
| 2-cycle (swap) | 4 opcodes, 2 temps  | 3 opcodes, 1 temp  |
| chain of 3     | 5 opcodes, 2 temps  | 3 opcodes, 0 temps |
| independent    | n opcodes, 0 temps  | n opcodes (tie)    |

The extra temps also raise register pressure right at jmp sites, which under spilling can trigger a
spill the general mover would avoid. So the mechanism is verified; the open question is *magnitude*.
Verification ladder:

1. **Unit.** The `jmp_block_params_parallel_move*` tests currently only assert the VM finishes;
   extend them to snapshot emitted opcodes and add chain/cycle cases, then diff inline vs a general
   prototype. Deterministic, no corpus.
2. **Crafted.** A loop that rotates ≥3 loop-carried variables makes each back-edge a chain/cycle;
   compile `--force-brillig` and count `mov`s (× trip count for the dynamic cost).
3. **Definitive.** Point `jmp_setup` at the revived solver and diff bytecode + execution steps
   across the corpus and protocol circuits. Zero diff ⇒ Phase 0 is purely a linear-scan
   prerequisite; shrinkage ⇒ a quantified standalone win.

Reviving it behind a general entry point (keeping the specialized fast path for returns) is small
and independently testable.

### Phase 0.5: extract a pluggable allocator seam

Today `BrilligBlock` *makes* the allocation decisions (when to spill, `lru_victim`,
`ensure_register_capacity` with hard-coded Ns) and `SpillManager` only does bookkeeping. To swap
strategies without rewriting codegen — and to A/B benchmark greedy vs linear scan behind a flag —
invert this: push the decisions behind an interface and leave `BrilligBlock` a pure consumer that
**emits whatever opcodes the allocator dictates**.

The unifying abstraction is a plan of actions: the allocator decides *where each value lives and
what to spill/reload/move*; the driver *emits* the resulting opcodes (and chooses where to place
edge code — predecessor exit, successor entry, or branch path):

```rust
// Codegen-facing addresses are registers (`MemoryAddress`); spill slots (`SpillSlot`) appear only
// inside actions, which the allocator fills in from its plan. Every action carries `value` so
// codegen updates its register shadow mechanically (Reload/Move -> value now at the destination
// register; Spill/Prune -> value leaves the shadow). All but `Prune` lower to an opcode; `Prune` is
// bookkeeping-only, keeping the shadow's registers exactly the occupied set as values die.
enum Action {
    Spill  { value: ValueId, from: MemoryAddress, to: SpillSlot },     // store a register to a slot
    Reload { value: ValueId, from: SpillSlot, into: MemoryAddress },   // load a slot into a register
    Move   { value: ValueId, from: MemoryAddress, to: MemoryAddress }, // register-to-register
    Prune  { value: ValueId, register: MemoryAddress },                // drop a dead value, freeing its register (no opcode)
}

trait Allocator {
    // The register-resident live-in values at this block's entry (value -> register). Codegen seeds
    // a fresh per-block register shadow from this. Spilled live-ins are not listed — they are
    // reloaded on first use via a self-contained Reload. Pure in `block`, so blocks may be emitted
    // in ANY order.
    fn begin_block(&mut self, block: BasicBlockId) -> Vec<(ValueId, MemoryAddress)>;

    // A value comes into existence at its def point — a constant, an instruction result, or an ABI
    // parameter. It is born in a register (Brillig computes into registers, never the heap) and the
    // per-instruction-fits invariant guarantees room, so this returns that register plus the spills
    // to free it. The driver then writes the value there. If the value's home is a spill slot, a
    // subsequent self-contained Spill stores it.
    fn define_variable(&mut self, value: ValueId) -> (MemoryAddress, Vec<Action>);

    // Reserve `scratch` free registers for the RAII temporaries codegen allocates while lowering an
    // instruction (signed-comparison scratch, array/call setup, ...), spilling to make the room and
    // returning those spills. The contract is "N slots are free", not "here are the registers" —
    // codegen owns the temporaries via the `Allocated` path. The count comes from
    // `instruction_scratch_demand`. Operands are made resident separately and lazily, one at a time,
    // via `use_variable` (see "Operand residency" below), so this is scratch-only and keys on a
    // count rather than an instruction id.
    fn reserve_scratch(&mut self, scratch: usize) -> Vec<Action>;

    // Make one already-defined value resident on demand and return the register it now occupies,
    // plus the actions to get it there: a Reload if it was spilled, and any Spills to free a slot
    // (empty if it is already resident). This is the single operand-residency primitive: the driver
    // calls it per operand as it reads them (`convert_ssa_value`), for every instruction — not only
    // streaming ones. A streaming op like `MakeArray` is then literally a loop over this — per array
    // element, retiring each working slot between iterations (RAII), so the reserved set stays at the
    // working-set size rather than the operand count (see "Streaming operands: MakeArray").
    fn use_variable(&mut self, value: ValueId) -> (MemoryAddress, Vec<Action>);

    // After lowering `inst`: retire the values whose last use it was, freeing their registers for
    // reuse later in the block. Returns a `Prune` for each retired value that held a register, so
    // codegen drops it from its shadow; no opcodes are emitted. The allocator owns liveness, so
    // codegen cannot know when a value dies — this is how it is told. Without it the shadow would
    // accumulate dead entries until the next `begin_block` reset.
    fn after_instruction(&mut self, inst: InstructionId) -> Vec<Action>;

    // Before a terminator: make its operands available (return values, or a jmpif condition).
    fn before_terminator(&mut self, block: BasicBlockId) -> Vec<Action>;

    // Cross-block reconciliation for one edge. In the shipped (fixed-home) form this returns a
    // `ParamHome` per destination parameter — the register to move the jmp argument into, or the
    // slot to store it to — and the driver emits the move/store (and, for a `JmpIf` then-edge,
    // guards them with the branch condition). Non-parameter values crossing the edge need no entry
    // here: with fixed homes a resident value is already in its one register, and a value reconciled
    // through its slot is spilled by `before_terminator` and reloaded on demand in the successor. The
    // `Vec<Action>` form (destination-valued Move/Reload, see "Resolution") is the interval-splitting
    // upgrade, needed only to keep a value resident across one out-edge while spilling it across
    // another; the base plan does not require it.
    fn resolve_edge(&self, pred: BasicBlockId, succ: BasicBlockId) -> Vec<ParamHome>;

    // The coalescing map (union-find groups) is a construction-time INPUT, not a method.
}

// Where a destination parameter lives on an edge, as reported by `resolve_edge`. The source
// (the jmp argument) is materialized separately by `use_variable`, which reloads it and reports
// its register, so a spilled-argument-into-spilled-parameter pass is `use_variable`'s Reload
// (slot -> register) followed by the Slot store below — Brillig has no memory-to-memory move, so
// the register is a mandatory intermediate regardless.
enum ParamHome {
    Register(MemoryAddress), // move the argument into this register
    Slot(SpillSlot),         // store the argument to this slot
}
```

Notes on the interface:

- **Codegen shadows only registers; spill slots never enter its state.** Because `Reload`/`Spill`
  carry both ends, codegen never needs to know a value's slot — the allocator, which owns the plan,
  fills it in. So codegen keeps only a per-block `value -> MemoryAddress` map for register-resident
  values (what it needs to emit Brillig opcodes — the role `RegisterState` plays today), seeded by
  `begin_block` and updated as it applies each action (`Reload`/`Move` put a value at the destination
  register; `Spill`/`Prune` remove it) and each `define_variable`. The "glorified `SpillState`" —
  slots, the LRU, spill decisions — stays entirely in the allocator; nothing about slots survives
  into codegen.
- **`define_variable` is the single "value into existence" primitive**, covering constants,
  parameters, and instruction results. It returns the register the value is born in (values are
  always computed into a register) plus the spills to free it; the driver owns the actual write (the
  `const` opcode, the instruction, or the ABI placement). This keeps a single source of truth — the
  driver already knows which values to create where (from `ConstantAllocation`, the parameter ABI,
  the instruction itself), so the allocator does not also dictate that.
- **`after_instruction` is the symmetric "value out of existence" primitive.** The allocator owns
  liveness (last-use sets), so codegen cannot see when a value dies; `after_instruction` is how it is
  told, once the instruction is lowered. It frees the retired values' registers for reuse later in
  the block and returns a `Prune` per value that held one, so the shadow's registers stay exactly the
  live set at every program point rather than accumulating dead entries until `begin_block`. `Prune`
  carries the freed register purely so the driver can assert the shadow agreed — a cheap check that
  the one-way allocator → shadow flow never drifted. This exactness is also load-bearing for the
  free-list question below: once the shadow is a faithful residency mirror, a linear-scan allocator
  need not own a mutable register pool at all — codegen can derive its scratch free list from the
  shadow's complement (union the currently-live scratch, which RAII already tracks). That lets the
  allocator's methods become read-only queries over a precomputed plan, which is the end state this
  seam is aiming at.
- **`begin_block` buys emission-order independence.** Without it the register shadow would have to be
  function-global, forcing a dominance-respecting emission order (a value's `define_variable` must
  run before any block reading it). `begin_block` hands each block its register-resident live-ins
  directly, so the per-block shadow is self-contained. Whether blocks may then be emitted in *any*
  order depends on the allocator: the stateless linear-scan impl answers `begin_block` for any block,
  so yes; the stateful greedy impl must still be driven in dominance order (see "Stateful greedy vs
  stateless linear scan" below). In the base fixed-home design `begin_block` returns each value's
  constant register; with **interval splitting** the entry register can differ from the def register
  (path-reconciled by `resolve_edge`), and `begin_block` returns that — the plan-based analogue of
  today's `set_allocated_registers` reseed.
- **`InstructionId` is not the position.** Where a method carries a position it keys on
  `InstructionId` (`after_instruction`) — a stable DFG id, meaningful to codegen, occurring at most
  once — and the allocator maps it internally to a `ProgramPoint` (`LiveIntervals` assigns points to
  block entries, instructions, *and* terminators). The id says *which* operation; the program point
  says *where* in the linear order. The one exception is `reserve_scratch`, which currently takes a
  bare count; a plan-based allocator that pre-schedules scratch would key it on the instruction's
  position instead.
- **Terminators and the entry.** `before_terminator` + `resolve_edge` cover all terminators: a
  `Return` uses only `before_terminator` (reload return values); a `Jmp` uses only `resolve_edge`
  (pass args to their params' homes); a `JmpIf` uses both (condition via `before_terminator`, arg
  passing via `resolve_edge`, guarded by the condition on the then-edge). `before_terminator` also
  carries the non-parameter cross-block reconciliation (see "Resolution"), so `resolve_edge` is
  purely about parameters. Entry-block parameters arrive at ABI-fixed slots, so they are **fixed
  intervals** (Wimmer's term for pre-colored intervals) the allocator must *honor* rather than
  assign — surfaced through `define_variable` returning their fixed location with no spill.

- **Operand residency is per-operand, not batched.** The design originally split instruction setup
  into a batched `before_instruction(inst)` — reload all operands *and* reserve scratch in one call —
  with `use_variable` as a streaming companion only for ops like `MakeArray` that consume operands
  one at a time. The implementation collapsed the two: the driver makes each operand resident lazily
  via `use_variable` as it reads it (`convert_ssa_value`) — the streaming path generalized to every
  instruction — and a scratch-only `reserve_scratch(n)` reserves room for the RAII temporaries. So
  there is no `before_instruction`: `reserve_scratch` is its scratch half and `use_variable` its
  operand half. A batched variant remains a possible optimization (one call, shared spill-store
  batching), but with bounded operand sets the lazy path was simpler at no measured cost. See
  [Streaming operands: MakeArray](#streaming-operands-makearray).
- **Greedy/LRU impl** computes actions online (today's behavior); `set_allocated_registers`/`detach`
  become its internals.
- **Linear-scan impl** runs a pass first and serves the precomputed plan.

The interface is best shaped by a real implementation rather than an anticipated one: the greedy
path is **extracted behind it first** (a pure refactor that keeps tests green), and the linear-scan
implementation is then added against the same trait. Codegen retains only the per-block **register**
shadow (`value -> MemoryAddress`) from the notes above — the role `RegisterState` plays today, which
it needs to emit Brillig opcodes. It does *not* shadow spill slots: because the `Reload`/`Spill`
actions carry both ends, the allocator supplies each slot, so the whole `SpillManager` — slot records
*and* decision state (LRU, slot allocation, spill activation) — moves into the allocator. The
friction points are where the greedy allocator reacts to something the linear-scan plan decides up
front (notably scratch demand); those are the parts to prototype before the interface is fixed.

**Retirement surfaces as `after_instruction` + `Prune`.** Freeing a value's register at its last use
(today `remove_variable`) emits no opcode — the allocator only returns the register to its free list
for the next `define_variable`. The *decision* is internal: the greedy allocator owns the liveness
(`last_uses`, moved out of `BrilligBlock`) and retires as it advances through the instructions it is
called on; the plan-based allocator bakes reuse into its static assignment. But the *fact* of the
death still has to reach codegen, because its register shadow would otherwise keep the dead value's
entry until the next `begin_block` reset — a within-block drift from true residency. So
`after_instruction(inst)` returns a `Prune` per retired value that held a register, and codegen drops
it from the shadow. This keeps the shadow's registers exactly the live set at every point, which is
what lets a later plan-based allocator stop owning a mutable pool entirely and have codegen derive
its scratch free list from the shadow's complement. Both impls emit `Prune`; only the greedy one also
mutates a real free list behind it. (`Prune` carries the freed register purely so codegen can assert
the shadow agreed — a cheap sync check on the one-way flow.)

The one death `after_instruction` does *not* cover — a terminator's `return`/`jmp` argument
registers, dead only *after* the terminator — is reclaimed by the next `begin_block`, which for the
greedy allocator also *resets* its occupied set to the new block's live-ins (the
`set_allocated_registers` reseed), dropping whatever the previous block left behind. That is
sufficient because nothing runs between a terminator and the next `begin_block` that needs those
registers: `jmp`/`jmpif` arguments are consumed by `resolve_edge` (so they cannot be freed earlier
anyway), and a `return` has nothing after it. So `begin_block` doubles as the greedy allocator's
implicit end-of-block for that tail. (This is also why greedy `begin_block` is `&mut` — it resets
state — whereas for the plan-based allocator it is a pure query.)

**Stateful greedy vs stateless linear scan.** This statefulness split is the crux of the two impls,
and it shows at the interface. The **linear-scan** allocator is effectively *stateless to query* —
every method is a pure lookup into the precomputed plan — so it can be called for any block, at any
point, in any order; that is what lets codegen emit blocks in any order. The **greedy** allocator is
*stateful and online*: its answers depend on the register/spill free lists, the `value -> register`
cache that `define_variable` builds up, and the LRU — all of which evolve as it runs. It must be
driven in a **dominance-respecting order** (today's reversed post-order: a value defined before any
block that reads it) and should **assert** it is not called out of order. For example, greedy
`begin_block(block)` must return the register-resident live-ins' *addresses*: it knows *which* values
are live-in from liveness, but their addresses exist only once those values were defined in
earlier-processed blocks (today `set_allocated_registers` reads them from the `ssa_value_allocations`
cache). A live-in with no cached address means the driver called it out of order — panic rather than
guess. So the seam is order-agnostic *only* with the stateless impl; the greedy impl carries the
current design's ordering requirement, now made explicit and asserted.

### Phase 1a: swap the victim heuristic (LRU to next-use)

A local change that needs the interval/next-use analysis but **not** the global-assignment
restructuring: keep the `SpillManager`, per-block reseed, and permanent spills, and only change
*which* value is evicted.

- The rule is **spill the active value whose next use is furthest** (Belady's MIN; Wimmer's
  refinement of Poletto's "furthest interval end"), ranking only the intervals that are *live at the
  current point* (Wimmer's **active** intervals). A value whose hole covers the current point is
  **inactive**: its register is already reusable there with no `Store`/`Load` — the hole mechanism
  above, not a spill — so it never enters the victim contest. The metric is emphatically *not* "how
  deep is the hole": from inside a hole the value is dead-forward on this path, so its next use is
  unreachable, not merely far. Only active intervals are candidates, because only they cost a store
  and a reload.
- **It reorders preferences without giving up any capability.** Like the greedy allocator, linear
  scan can still spill a *live* value at any program point under pressure (via interval splitting) —
  spilling a needed value is its least-preferred option, not something it is forbidden to do, and it
  is not restricted to spilling only at holes or block boundaries. So nothing is given up relative
  to LRU; next-use just picks a better victim, fixing LRU's blind spot (evicting a not-recently-used
  value that is about to be used). For a uniform-cost straight-line sequence next-use is provably
  optimal; LRU is a worse approximation of the same thing.
- **The real headroom is cost-awareness.** Belady assumes uniform miss cost and a known linear
  future; we have neither (loop-weighted opcodes, branchy CFGs). So weight next-use distance by loop
  depth / execution frequency, so a hot-loop value is not evicted in favor of a cold one. That beats
  both LRU and naive next-use. The victim heuristic is a tuning surface, not a solved problem.

- **Data-structure cost — and a cheaper first step.** Furthest *next-use* is position-dependent, so
  it needs a per-value ordered list of use positions (a byproduct of `BUILDINTERVALS`, which already
  visits every use) plus a cursor advanced as the monotonic scan passes each use; the victim is the
  max over the active set (bounded by the register count), or a next-use-keyed max-structure
  re-keyed per use (O(uses · log R)) if the linear active scan is too costly at large frames.
  Poletto's furthest *interval end* avoids the position query entirely: the end is `last_use`, a
  single scalar already present in `LiveInterval` — no new data. It is a worse heuristic (it will
  evict a value used at the very next instruction if that value merely lives long, forcing an
  immediate reload), but it is free with today's representation and already beats LRU. So Phase 1a
  can itself be staged: furthest-`last_use` first (no new data); then a **block-granular** next-use
  — block granularity is a fine estimate for the *cross-block* tail (reusing per-block liveness, the
  exact instruction in a far block barely changes the ranking), but the *current* block needs
  instruction-level "remaining uses ahead of the current point," which is free since it is being
  walked (pure block granularity can't tell a current-block use *behind* the point from one *ahead*,
  and would falsely protect a value whose local uses have passed); then full instruction-granular
  next-use; then cost-aware weighting (a loop-depth weight per use position) — each measurable on the
  sweep.

Landing this alone measures the *victim-quality* component of the win in isolation from resolution.
If the sweep barely moves, the win is almost entirely in resolution — which sharpens where to
invest.

### Phase 1: interval-set liveness, linear-scan assignment, resolution

**Assignment.** Give each interval a register for its lifetime; two intervals separated by a hole
are non-interfering and so may share one (the diamond example above — `w`, live only in `b1`, shares
`v`'s register) — that is how holes are exploited, with no memory traffic.
Only when even that leaves more simultaneously-live values than registers does a *pressure* spill
occur: evict a live (active) interval by the cost-aware next-use rule above, optionally splitting it
so only part of its range lives in memory.

**Spilling under a global assignment.** A value has a **single spill home**, decided globally — not
a fresh slot per spill event. So a spilled value occupies one slot consistently on all paths, and
its successors see it in the same place: **no cross-edge reconciliation for a purely-spilled
value**, and it is reloaded only where it is actually used (or at a split point), not at every
boundary. This is exactly why `permanent`/transient **dissolves**: the current design invented
permanent slots to give cross-block spills one canonical location (per-block spilling would pick
different slots on different paths and be inconsistent); a global assignment does that uniformly,
and the never-freed-slot leak (noir-lang/noir#11695) goes away with it.

Spill slots are a **symmetric** resource to registers: non-overlapping values share a slot exactly
as they share a register, and slot assignment is the same interval-packing problem (the current
`SpillManager` already does this for transient slots via its free list and `max_spill_offset`). The
only asymmetry is *access cost* (a register is a free operand; a spill slot needs `Load`/`Store` —
see "Not in scope" below) and *supply* (registers are a fixed budget, slots elastic). One can model
registers + slots as a single location pool where registers are the cheap locations.

**Resolution** is **edge-centric**. `resolve_edge(pred, succ)` computes what a specific edge needs;
reconciliation is required *only where a value's location differs across a merge* (register on one
predecessor, spilled — or, under interval splitting, a different register — on another). When it must
reconcile, it chooses the direction that wastes fewer registers: e.g. store-to-match on the
in-register edge rather than reload-into-a-register early and tie that register up through blocks
where the value is idle.

**What the shipped `resolve_edge` returns, and why non-param values need no entry.** The extracted
greedy allocator (Phase 0.5) returns a `ParamHome` per destination parameter — `Register(r)` to move
the argument into, or `Slot(s)` to store it to — and nothing else. That is complete because of the
**fixed-home** rule: each value has one register `R_v` and one slot `S_v` for its whole life, so a
non-parameter value crossing an edge is reconciled by *state*, not *location*, with only four cases:

| pred exit → succ entry | edge action |
|---|---|
| `R_v → R_v` | none — same register (the point of fixed homes) |
| `S_v → S_v` | none — same slot |
| `R_v → S_v` | store `v` to `S_v` |
| `S_v → R_v` | reload `v` from `S_v` |

The two non-trivial cases are carried by **`before_terminator`** — which already returns `Vec<Action>`
for exactly this, spilling cross-block live-ins — and by on-demand **`use_variable`** reloads in the
successor, *not* by `resolve_edge`. So `resolve_edge` never has to name a non-parameter value: the
source side of a parameter pass is materialized by `use_variable` (which reports the argument's
register, reloading it if it was spilled), and everything else crossing the edge already sits in its
one fixed home. `ParamHome` is therefore a correct, stable interface for the base fixed-home plan.

The **one** thing it cannot express is keeping a value *resident across one out-edge of a branching
block while spilling it across another* — a per-out-edge, non-parameter decision a per-*block*
`before_terminator` cannot make. That is the interval-splitting optimization, and it is the sole
reason to upgrade `resolve_edge` to the destination-valued `Vec<Action>` form described next. The
correctness-preserving fallback is always available — spill uniformly in `before_terminator` and let
the successor reload (what greedy does) — at the cost of a reload the resident edge did not need.

*The remainder of this section describes that `Vec<Action>` upgrade: per-edge resolution of arbitrary
(including non-parameter) values, which subsumes the split-residency case.* The merge-entry location
is a plan decision, and the "picture of both predecessors" lives in the
plan rather than being reconstructed at codegen time. The RPO construction pass processes every
predecessor before the merge, so it knows the value's location at each predecessor's exit and picks
the entry home once; `resolve_edge(pred, succ)` is then a per-edge lookup comparing that
predecessor's exit to the chosen entry. So if `v` reaches a merge in a register from one predecessor
and spilled from another, and the entry home is the spill slot, the in-register edge stores `v` and
the already-spilled edge needs nothing — decided from the plan, with no cross-predecessor inspection
during codegen (which a greedy allocator would require).

**Resolution actions are destination-valued.** At an edge the value that ends up in a register is the
*destination param*, sourced from the *argument*'s current location — the phi makes them the same
runtime value. So each action names the param as its `value`: a resident argument becomes a `Move`
into the param's register, and a *spilled* argument becomes a `Reload` that loads the argument's slot
**straight into the param's register** (no intermediate). Because `value` is the param, the shadow
would read the param at its own home, never "the argument now lives in the param's register" — and it
is moot regardless, since `begin_block(succ)` reseeds the shadow on entry.

For example, at `jmp b3(v1, v2)` with target params `b3(p1, p2)`, where the plan puts `p1` in `R2`
and `p2` in `R5`, `v1` is resident in `R1`, and `v2` is spilled at slot `S`, `resolve_edge` returns:

```
Move   { value: p1, from: R1, to: R2 }    # resident arg v1 -> param p1's register
Reload { value: p2, from: S,  into: R5 }  # spilled arg v2 loaded directly into param p2's register
```

Both name the *param*, so the resulting shadow is `p1 @ R2`, `p2 @ R5` — the params at their homes.
If the param registers instead overlapped the argument registers in a rotation (a swap), the
allocator sequences the `Move`s with a scratch register it reserves — the parallel-copy job the
revived Phase 0 solver performs. This is why `Move` stays an action rather than being synthesized by
codegen: the allocator owns the cycle-break scratch (a register decision), and interval splitting
produces single-value register-to-register `Move`s of its own. The allocator therefore *uses* the
Phase 0 solver to produce the ordered `Move`/`Reload` sequence; codegen still only emits it.

The non-spilling path *already* does edge resolution for block parameters (via
`codegen_mov_registers_to_registers`); Phase 1 generalizes it to all cross-block values. Framed
this way the change is "make the spilling path behave like the non-spilling path, plus eviction only
where forced," not a new phase from scratch.

**Placement of edge code is the driver's job, and needs no new IR blocks.** The only multi-successor
terminator is `JmpIf` (two successors); `Jmp` has one, `Return` none. So a critical edge always has
a `JmpIf` at its tail, and a `JmpIf`'s two branches are already distinct stretches of bytecode. The
driver places an edge's actions:

- at the predecessor's exit, if it has one successor (e.g. a `Jmp`);
- at the successor's entry, if it has one predecessor;
- otherwise (critical edge) on the branch-specific bytecode path — either emit after the branch on
  the taken path, or **predicate** by the condition. The `jmpif` lowering already predicates
  then-arg stores (`codegen_conditional_spill_store`), so the mechanism is partly built. For general
  register↔register resolution moves, branch-path emission is cleaner than building conditional
  variants of every op.

So "split the critical edge" from the literature maps onto our model as "emit on the branch-specific
path (or predicate it)" — a bytecode-layout concern, not an IR-block one.

## Cross-cutting concerns

These are not phases but concerns that span the design; each explains how a piece fits, rather than
adding a delivery step.

### Coalescing

Coalescing eliminates the move at a jmp by having an argument and its destination block parameter
share a register. Today it is a *separate pre-pass* — `CoalescingMap`, built with union-find into
connected components of values that should share a register, which the greedy codegen then honors —
and it is **mutually exclusive with spilling**: the spill manager disables it because it cannot
reason about pre-shared registers (see noir-lang/noir#11909, #12256). Note it does not itself
allocate; it only records "when defining x, reuse y's register."

Under the interval allocator this stops being a separate, exclusive pass: **coalescing is interval
merging.** Two copy-related, non-interfering values are assigned the same register, so the
resolution move on that edge is a self-move and is elided (the "often zero moves" case above). The
existing `CoalescingMap` connected components are exactly the merge groups, so they become a
**construction-time input** to the allocator (a register-preference / merge hint), not a rival
strategy. Coalescing and spilling then coexist by construction — coalescing is a preference during
assignment, spilling is eviction under pressure, one scan does both — which *resolves* the mutual
exclusion tracked in #11909/#12256 rather than working around it. When a coalesced group does
interfere, the allocator falls back to distinct registers plus the resolution move, no worse than
today.

This split falls out of the phasing. Phase 0.5's greedy `Allocator` **preserves today's mutual
exclusion internally** — it encapsulates both the `CoalescingMap` and the `Option<SpillManager>`,
mode-gated by `needs_spill_support` exactly as `FunctionContext::new` is today (spilling on ⇒
coalescing map empty), so extraction stays a pure, behavior-preserving refactor. The linear-scan
`Allocator` then **dissolves** the exclusion via interval merging. Both sit behind the same trait
with the `CoalescingMap` as a construction-time input, so moving from one to the other is an impl
swap — no change to the driver or the trait.

### Temporaries and procedures

Codegen allocates registers that SSA liveness never sees. Two disjoint cases:

- **Separately-compiled procedures** (`array_copy`, `mem_copy`, the vector ops) are compiled once
  against `ScratchSpace` (Direct addressing) and invoked by a procedure call. Their temporaries live
  in scratch space, never in the calling function's stack frame, so the allocator has nothing to
  reserve for them. They keep manual RAII allocation and are out of scope for the pass.
- **Inlined codegen temporaries** (e.g. `codegen_make_array`'s four registers, signed-comparison
  scratch, store-address scratch, call-argument setup) are lowered into the SSA function's frame and
  *do* compete with SSA slots. Today they are guarded reactively by `ensure_register_capacity`, which
  LRU-spills to make room.

The allocator absorbs the inlined case by **modelling each op's temp demand as short fixed intervals
at that instruction point**. The per-instruction count already exists as `instruction_scratch_demand`;
its consumer changes from a scalar added to `max_live_count` to an *interval generator* that
synthesizes that many short intervals (def and last-use at the same point). Linear scan then
guarantees enough slots are unassigned by any SSA value live *at that instruction*, pre-scheduling a
spill only if that point is genuinely oversubscribed. This is a per-point reservation, not a
globally reserved register: distinct instructions may place their temporaries in different physical
slots, and slots not needed for scratch at a point stay available to SSA values there — which is why
it beats reserving the top N slots globally.

The temporary draws from the free-at-this-point set — a read-only consequence of the assignment
(`free(P) = all_slots \ { slot(v) : v live at P }`), not a mutable per-block free-list — so
`set_allocated_registers` and `detach` go away, replaced by a static occupancy query plus in-scope
RAII (temporaries never needed `detach`). Because scratch demand becomes an exact interval rather
than a scalar added to a lower-bound peak, the `SPILL_MARGIN` fudge on `max_live_count` can shrink or
retire (to the extent `instruction_scratch_demand` attributes every transient). The coupling that
the analysis must know each lowering's temp appetite remains; an assertion tying the reserved count
to the actual allocation would guard against drift.

This is where the driver's register shadow earns a second role, and why `Prune` was worth adding in
Phase 0.5. The plan-based allocator serves read-only queries and does not own a mutable register
pool — but `BrilligContext` still needs one to hand out temporaries (`allocate_register`). The
intent is to implement the `RegisterAllocator` trait *directly on the shadow*: the shadow's occupied
registers are exactly `{ home(v) : v live at P }`, so a temporary is drawn from the complement (minus
any scratch already live, which the `Allocated` RAII tracks) and returned on drop. This is sound only
because `Prune` (returned by `after_instruction`) keeps the shadow an exact residency mirror within a
block — without it the shadow would retain dead entries and the free set would be wrong. So the
Phase-0.5 pieces (one-way shadow + `Prune`) are precisely what lets the Phase-1 allocator drop its
pool and let codegen source temporaries from the shadow.

**Constants** are handled the same way, and are already partly modelled. `ConstantAllocation`
decides a materialization point per constant — the common dominator of its uses, hoisted out of
loops where possible — and the greedy pass materializes it there (loads it into a register), treats
it as a value defined at that point, and frees it at last use; each function re-materializes its own
constants unless they are hoisted into global memory by a separate pass. So a constant gets a
synthetic interval `[materialization point, last use]`, exactly like a temporary but with the def
point supplied by `ConstantAllocation` — and `live_intervals.rs::adjust_constant_defs` already pulls
constant defs back to their allocation-block entry.

### A single instruction must fit in registers at once

An instruction's inputs, results, and scratch are all live at its program point, so they must be
register-resident *simultaneously* — no allocator can spill around it, because you cannot evict an
operand you are about to read (and multiple results must not evict one another). This is a hard
**per-instruction lower bound on the frame size**.

Today nothing checks it. `max_live_count + SPILL_MARGIN >= max_stack_frame_size` only *toggles
spilling on*; the sole general enforcement is the reactive `"Stack frame too deep"` panic in
`Stack::allocate_register`, which fires mid-codegen once an allocation actually overflows (calls are
the one exception — `codegen_call` explicitly checks `"Call arguments would exceed stack frame
bounds"`). So the invariant holds today only because the default frame (2048) dwarfs any real
instruction's peak; shrink it far enough and the panic, not a diagnostic, is what you get. Current
codegen also relies on this fit implicitly: it protects operands by *recency* — reading an operand
touches it to most-recently-used and block live-ins are seeded as least-recently-used, so LRU
eviction reaches for older values first — which works only while the frame exceeds the peak.

The plan-based allocator makes this safe *by construction* and *checkable*:

- The pressure at the instruction's point counts inputs + results + scratch together (as
  `max_live_count` already does), so room for all of them is reserved at that point — operands via
  `use_variable`, scratch via `reserve_scratch`; `define_variable` for each result then draws a
  pre-reserved slot and never evicts a sibling.
- The furthest-next-use victim rule *cannot* evict them: their next use is immediate, so they are
  never the furthest — protection by definition, not by the LRU recency proxy.
- The pass should *assert* the bound up front, turning the latent panic into a clear "instruction
  needs N registers, frame is M". This landed on its own ahead of the allocator rewrite
  (noir-lang/noir#13306) as `VariableLiveness::min_live_count`. Two refinements fell out of
  *measuring* the real codegen floor (a frame-size sweep, smallest frame each instruction compiles
  in without the panic):
  - The bound is `max(inputs, results) + scratch`, not `inputs + results + scratch`: a result reuses
    an operand register in place (the binary opcode reads both sources before writing the
    destination, and a live operand it overwrites is first preserved in a spill slot). For a
    scratch-bearing op the sweep confirmed the exact floor *is* `inputs + results + scratch`
    (the result does not reuse when a post-op check keeps operands and result live together), so
    `max(inputs, results) + scratch` is a deliberate *lower* bound — under-counting by up to
    `results`. That is the safe direction for an assertion: a false positive would reject a frame
    codegen could handle, whereas an under-count merely falls through to the existing panic.
  - It is compared against the *usable* slots, `frame - Stack::START_OFFSET`, not the raw frame.
  - "inputs" means the operands that must be resident *simultaneously*. A streaming instruction reads
    its operands one at a time, so its floor is far smaller than its operand count — see
    [Streaming operands: MakeArray](#streaming-operands-makearray).

### Streaming operands: MakeArray

`MakeArray` violates the "operands must be resident simultaneously" premise above, and it is the one
place that bites in practice. Its elements are written to the heap **one at a time**
(`codegen_make_array` -> `initialize_constant_array`); even the repeating-item runtime loop
materializes only a single item's subitems (`subitem_to_repeat_variables`). So however large the
literal, the simultaneously-resident element working set is `typ.element_types().len()` (1 for a
scalar array, tuple-arity for a tuple array), never the element count.

Counting every element as a simultaneous input over-states the floor badly enough to be a real bug,
not a hypothetical: it made `execution_success/brillig_large_array` fail to compile as soon as the
floor assertion landed. The fix (noir-lang/noir#13306) is `instruction_min_inputs`, which
special-cases `MakeArray` to `element_types().len()`. Its floor is then
`max(element_working_set, results) + scratch`, with `scratch = 3` for `MakeArray`:
`codegen_make_array` reserves `ensure_register_capacity(4)` = the result array-pointer + `items_pointer`
+ `write_pointer` + a temp, and the result is an SSA value counted separately. `MakeArray` is the
only heap-streaming operand case; `Call` marshals many arguments too, but into contiguous,
`codegen_call`-bounds-checked *frame* slots, so counting those is correct.

**Interval-model consequence (recover this when implementing linear scan).** Unlike genuine scratch
(born and dead at a single point), array elements are real SSA values with homes, consumed at
*staggered* moments. With instruction-level program points their `last_use` all collapse onto the
one `MakeArray` point, so naive `[def, last_use]` intervals make all N overlap — the interval-space
restatement of the same over-count. Two ways to model the streaming:

- **(a) Sub-instruction program points.** Give `MakeArray` one micro-point per element store; each
  element's interval ends at *its* store. Overlap at any micro-point is the working set (+ result +
  temps), so pressure falls straight out of interval overlap with no special reuse logic — linear
  scan shares one physical slot across the staggered elements, and an element that happens to be
  *already* resident is stored without a reload. Cost: `ProgramPoint` gains sub-instruction
  granularity for streaming ops. The more faithful — and more elegant — model.
- **(b) Reserved-reused synthetic slots + on-demand reload.** Keep points coarse; reserve
  `element_types().len()` working slots at the `MakeArray` point as short intervals (like scratch),
  and have the driver RAII-cycle them across the N stores, pulling each element in via an on-demand
  `use_variable(value) -> (MemoryAddress, Vec<Action>)` that reloads from the element's home only if it is not already
  resident. The synthetic slots are *destinations* for on-demand reloads, not the elements' own
  lifetimes — a per-element interval would recreate the N-wide over-count. This localizes the special
  case to streaming ops, at the cost of the on-demand discipline — though `use_variable` is a
  first-class allocator method regardless, paired with the reservation-style `reserve_scratch` (whose
  contract is "K slots are free", not "here are the registers").

The two are duals: (a) expresses the reuse as non-overlapping intervals sharing a slot; (b) as one
reservation the driver fills imperatively. Either way the room reserved at the `MakeArray` point
equals the floor the assertion enforces (result array-pointer + element working set +
`items_pointer`/`write_pointer` temps), so analysis and allocator agree by construction. This is also
why the implementation makes operands resident per-operand rather than in a batch: every instruction
— bounded or streaming — drives `use_variable` for each operand as it is read, so a streaming
instruction needs no up-front enumeration of its operands at all.

## Relationship to noir-lang/noir#11638

This design starts from the linear-scan tracking issue #11638 and largely follows it, but sharpens
or diverges on several points worth surfacing:

- **Holes are a prerequisite, not a "later" feature.** #11638 sketches an incremental rollout with
  lifetime holes as a later addition. But single-interval linear scan over-estimates register
  pressure — `max_live_count` is *already* hole-aware — and would *regress*, spilling where the
  current greedy+LRU fits. The floor is block-granular holes (see "Holes are required").
- **Coalescing and spilling coexist by construction.** They are mutually exclusive today
  (#11909, #12256); under the unified allocator coalescing is interval merging (a construction-time
  input), so both are active at once — this *resolves* those issues rather than working around them.
- **The goal is spill-*volume* reduction via stable homes + resolution**, not #11638's "a value can
  live in a register for part of its life and a spill slot for another" — the greedy LRU already does
  that. Stability is the lever; splitting is a secondary one.
- **Splitting is more valuable in Brillig than the ticket implies.** With no memory operands, every
  use of a spilled value is an explicit `Load`, so keeping a register copy across a run of uses
  (splitting) saves real opcodes — not just a code-quality nicety.
- **The allocator seam must be position-indexed** to be plan-based and testable — #11638 flags the
  allocator/codegen entanglement; the `Vec<Action>` trait keyed by structural point is the concrete
  decoupling, and it must also cover non-instruction points (block entry, terminators) and
  materialization, which the naive shape misses.
- **"A spilled interval stays on the stack for its lifetime" is the Poletto model** — "no persistent
  register assigned," not "never in a register" (uses still reload transiently). Correct, but the
  `Load`-per-use point above is why we care about splitting.
- **The non-spilling path already has stable homes.** #11638's framing of the entanglement can read
  as if all allocation is reshuffled per block; in fact non-spilling values keep their cached
  register, and only spilling loses stability. The win is therefore concentrated in spill-heavy code.

## Validation

Linear scan is a heuristic retargeted to an opcode/proving-cost objective it was not designed for,
so the win must be measured, not assumed. With today's generous fixed frame
(`MAX_STACK_FRAME_SIZE = 2048`) almost no real program spills, so the spill path is not exercised by
the corpus at default settings — which is why `LayoutConfig` / the `--max-stack-frame-size` flag and
the `generate_brillig_small_stack_execution_success_tests` harness exist: they shrink the frame (the
harness uses `--force-brillig --max-stack-frame-size 64`) to force spilling deterministically.

The proposed benchmark is a **frame-size sweep** over a descending ladder (e.g. 2048 → 512 → 128 →
32 → 16 → 8): compile the corpus and the noir-protocol-circuits at each size and compare emitted
bytecode before and after the allocator change, per program per size.

- **The protocol circuits are the load-bearing part**, not the toy corpus: at 2048 the corpus barely
  spills, so the win only shows on programs large enough to spill at realistic frames.
- **Static bytecode size is a proxy; loop-weighted dynamic opcodes are the truer cost.** A spill in a
  hot loop costs far more than its one static opcode. The `execution_success` programs run, so
  capture VM step counts too. Note that CI's `compare_brillig_execution_reports` already measures
  executed-opcode counts (on the internal corpus), and `external_repo_reports` already *executes* the
  protocol circuits with inputs (timing only) — but neither sweeps `--max-stack-frame-size`, so the
  harness fills exactly that gap: deterministic executed-opcode counts on the protocol circuits
  across a frame-size ladder.
- **Report the transition, not the mean.** Flag any program that changes between success and failure
  across the ladder, and locate each program's spill onset relative to its peak pressure — that
  boundary is where the allocator earns its keep, and it quantifies how tightly frames can be shrunk
  before spill cost outweighs the per-call frame-copy saving.

### Not in scope: in-place spill operands

A textbook target can use a spilled value directly as an ALU operand (`add eax, [ebp-8]`) because its
spill area is statically frame-relative and the ISA has base+displacement memory operands. Brillig
cannot: the spill region lives on the heap behind a per-frame runtime base pointer (`sp[1]`), and
Brillig opcodes take only statically-encoded `Direct`/`Relative` addresses — only `Load`/`Store`
dereference a runtime pointer. Addressing spill slots in place would need a new heap-base-relative
addressing mode, i.e. a VM-semantics change requiring AVM buy-in. It is recorded here only as a
contingent future option; every phase above depends on nothing outside `noirc_evaluator`.

## Delivery checklist

PR-sized increments, ordered to land value early. Top-level boxes are roughly one PR each unless
broken down; nested boxes are the sub-PRs of a multi-PR phase. Boxes within a phase are ordered by
dependency.

- [x] **Per-instruction floor assertion** — standalone, independent of everything below
      (noir-lang/noir#13306). Landed as `VariableLiveness::min_live_count`, asserting
      `max over instructions of (max(inputs, results) + scratch) ≤ frame - Stack::START_OFFSET` up
      front, converting the reactive "Stack frame too deep" panic into a clear diagnostic. See
      [A single instruction must fit](#a-single-instruction-must-fit-in-registers-at-once) for the
      `max(...)`/usable-slots refinements and [Streaming operands](#streaming-operands-makearray) for
      the `MakeArray` exception.
- [x] **Phase 0 — general parallel moves** (noir-lang/noir#13307)
  - [x] Revive the any-to-any parallel-move solver behind a general entry point. Rather than keeping
        a separate specialized path, #13307 lifted the consecutive-destination restriction
        `10305` had added to `codegen_mov_registers_to_registers`, so the one routine now serves both
        return-value copies and general moves (and also takes an optional `condition`, emitting
        `conditional_mov`).
  - [x] Point `jmp_setup` at it, replacing the inline temp-per-conflict mover.
  - [x] Opcode-snapshot test for cycles: a block that rotates its parameters now needs just 1
        temporary, not one per parameter.
- [ ] **Phase 0.5 — pluggable allocator seam**
  - [ ] Define the `Allocator` trait and `Action` type.
  - [ ] Extract the current greedy path (cache, `CoalescingMap`, `Option<SpillManager>`, decisions)
        behind the trait as a behavior-preserving refactor; `BrilligBlock` becomes a driver holding
        the register shadow. Tests stay green.
  - [ ] Flag to select the allocator implementation (enables the A/B benchmark).
- [ ] **Phase 1a — interval-based victim heuristic** (greedy spilling, smarter victim)
  - [ ] Activate `LiveIntervals` in codegen (today `#[cfg(test)]`) and compute register pressure for
        real.
  - [ ] Furthest-`last_use` victim (no new data beyond the interval envelope).
  - [ ] Furthest next-use victim (add per-value use-position lists + cursor).
  - [ ] Cost-aware (loop-weighted) next-use.
- [ ] **Phase 1 — linear-scan allocator**
  - [ ] Interval-set liveness (block-granular holes): keep the range list instead of collapsing to
        `[def, last_use]`.
  - [ ] Global assignment pass producing the plan (register/slot per value; per-point action tables).
  - [ ] Constants and inlined temporaries as synthetic intervals (`ConstantAllocation`,
        `instruction_scratch_demand`).
  - [ ] Resolution phase (`resolve_edge`) via the Phase 0 parallel move, with edge placement
        (predecessor exit / successor entry / branch path).
  - [ ] Linear-scan `Allocator` implementation behind the trait; coalescing via interval merging
        (dissolves #11909/#12256).
  - [ ] Interval splitting (register-part / spill-part) + `begin_block` reseed of split entry homes.
- [ ] **Validation** (cross-cutting)
  - [ ] Frame-size sweep harness over the corpus + noir-protocol-circuits, reusing
        `compare_brillig_execution_reports`; report per-program bytecode/step deltas and the
        success-vs-failure and spill-onset transitions across the `--max-stack-frame-size` ladder.

Out of scope, contingent on AVM buy-in (not tracked here): in-place spill operands (a heap-base-relative
addressing mode).
