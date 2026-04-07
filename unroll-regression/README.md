# Brillig Unroller Force-Unroll Regression

## Summary

The Brillig loop unroller's cost model has a blind spot: when `useful_cost`
drops to zero, `unrolled_cost = useful_cost * iterations` is always zero
regardless of iteration count. This causes the `force_unroll` check to
approve arbitrarily large loops.

In practice this was exposed when removing a handful of `inc_rc`
instructions (via a frontend clone-elimination optimisation). The removal
dropped `useful_cost` from 3 to 0 for a 754-iteration scalar-multiplication
loop in `noir_bigcurve`'s mnt6_753 curve, causing the unroller to fully
unroll it and producing a ~300k-line function from a ~5k-line input.

## Bug location

`compiler/noirc_evaluator/src/ssa/opt/unrolling.rs`, in
`should_unroll_in_brillig`:

```rust
let force_unroll = s.unrolled_cost() <= force_unroll_threshold;
```

`unrolled_cost()` is `useful_cost() * iterations`. When every instruction in
the loop body is classified as "useless" (constant-foldable after
unrolling), `useful_cost` is 0, so `unrolled_cost` is 0 for any iteration
count. The guard that is supposed to prevent over-aggressive unrolling is
completely bypassed.

The existing `conservative_unrolled_cost()` (which does NOT subtract
`useless_cost`) correctly reports 1,400,178 for this loop, but it is only
consulted by `is_small()`, not by the `force_unroll` path.

## Concrete numbers

| Metric | Master (with inc_rc) | Branch (without inc_rc) |
|--------|---------------------|------------------------|
| `useful_cost` | 3 | 0 |
| `unrolled_cost` | 2,262 | 0 |
| `conservative_unrolled_cost` | 1,404,702 | 1,400,178 |
| `force_unroll` | false | **true** |
| Unrolling decision | NOT unrolled | **UNROLLED** |
| Post-unroll f8 lines | 5,158 | 312,297 |
| Unrolling pass time | 18 ms | 68,161 ms |

## Reproduction

The two `.ssa` files in this directory are the full program SSA captured
just before the second `Unrolling` pass (step 51 in `noir-ssa list`).

```bash
# Fast — master (with inc_rc): 0.1s, 16k lines output
noir-ssa transform -p "Unrolling" -s unroll-regression/master.ssa -o /dev/null

# Slow — branch (without inc_rc): 69s, 324k lines output
noir-ssa transform -p "Unrolling" -s unroll-regression/branch.ssa -o /dev/null
```

The only meaningful difference between the two inputs is that `branch.ssa`
is missing ~16 `inc_rc` instructions in `hash_to_curve_inner` (f8). This
drops the 754-iteration loop's `useful_cost` from 3 to 0.

## Suggested fix

The `force_unroll` path should also guard against unreasonable total
expansion. Options (not mutually exclusive):

1. **Cap `force_unroll` by `conservative_unrolled_cost`**: this already
   exists and is the pre-folding estimate. A check like
   `s.conservative_unrolled_cost() <= some_limit` would catch this case.

2. **Floor `useful_cost` at a small per-iteration overhead**: even a fully
   constant-foldable loop still costs *something* to materialise and then
   fold. A floor of e.g. 1 per iteration would make the 754-iteration loop
   report `unrolled_cost = 754`, which exceeds the threshold.

3. **Cap the iteration count for `force_unroll`**: a direct
   `s.iterations <= max_force_unroll_iterations` guard. The current
   `max_unroll_iterations` is already checked, but it applies to all
   unrolling paths equally and is set high enough to allow this loop.
