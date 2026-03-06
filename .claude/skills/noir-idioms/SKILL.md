---
name: noir-idioms
description: Guidelines for writing idiomatic, efficient Noir programs. Use when writing or reviewing Noir code.
---

# Writing Idiomatic Noir

These guidelines help you write Noir programs that are readable, idiomatic, and produce efficient circuits.

## Core Principle: Hint and Verify

Computing a value is often more expensive in a circuit than verifying a claimed value is correct. Use `unconstrained` functions to compute results off-circuit, then verify them with cheap constraints.

```noir
// Expensive: sorting an array in-circuit requires many comparisons and swaps
let sorted = sort_in_circuit(arr);

// Cheaper: hint the sorted array, verify it's a valid permutation and is ordered
let sorted = unsafe { sort_hint(arr) };
// verify sorted order and that sorted is a permutation of arr
```

Note that the compiler already injects unconstrained helpers for some operations automatically (e.g., integer division). Don't hint what the compiler already optimizes — focus on higher-level computations like sorting, searching, and array construction where the compiler cannot automatically apply this pattern.

### What to Hint

Hint the **final result**, not intermediate values. If your unconstrained function computes helper structures (masks, indices, accumulators) on the way to an answer, return only the answer and verify it directly against the inputs. Fewer hinted values means fewer constraints needed.

### Safety Comments

Every `unsafe` block must have a `// Safety:` comment that explains *why* the constrained code makes the hint sound — not just *where* the verification happens, but what property it enforces:

```noir
// Safety: each result element is checked against a[i] or b[i] depending on
// whether i <= index, so a dishonest prover cannot substitute arbitrary values
let result = unsafe { my_hint(a, b, index) };
```

## ACIR vs Brillig: Different Optimization Goals

Noir compiles to two different targets depending on context, and they have fundamentally different performance characteristics.

**ACIR (constrained code)** — the default. Every operation becomes arithmetic constraints in a circuit. Optimize for **gate/constraint count**: fewer constraints = faster proving.

**Brillig (unconstrained code)** — functions marked `unconstrained`. Runs on a conventional VM. Optimize for **execution speed and bytecode size**: familiar performance intuitions apply.

### Key Differences

| Aspect | ACIR (constrained) | Brillig (unconstrained) |
|---|---|---|
| Loops | Fully unrolled — bounds must be comptime-known | Native loop support — runtime-dynamic bounds are fine |
| Control flow | Flattened into conditional selects — both branches are always evaluated | Real branching — only the taken branch executes |
| Function calls | Always inlined | Preserved when beneficial |
| Comparisons | Inequality (`<`, `<=`) requires range checks (costs gates) | Native comparison instructions (cheap) |

### Branching with `is_unconstrained()`

When a function may be called in either context, use `is_unconstrained()` to provide optimized implementations for each target. This is common in the standard library:

```noir
pub fn any<Env>(self, predicate: fn[Env](T) -> bool) -> bool {
    let mut ret = false;
    if is_unconstrained() {
        // Brillig path: use the actual length directly
        for i in 0..self.len {
            ret |= predicate(self.storage[i]);
        }
    } else {
        // ACIR path: iterate the full static capacity, guard with a flag
        let mut exceeded_len = false;
        for i in 0..MaxLen {
            exceeded_len |= i == self.len;
            if !exceeded_len {
                ret |= predicate(self.storage[i]);
            }
        }
    }
    ret
}
```

The constrained path must iterate the full `MaxLen` because ACIR loops are unrolled at compile time — the compiler needs a static bound. The unconstrained path can loop over exactly `self.len` elements because Brillig supports runtime-dynamic loop bounds.

### Practical Guidelines

- **Constrained code**: minimize constraints — use hint-and-verify, avoid unnecessary comparisons, leverage type-system range guarantees to simplify constraints.
- **Unconstrained code**: write for clarity and speed like normal imperative code — use dynamic loops, early returns, and mutable state freely.
- Don't add constraint-style verification inside `unconstrained` functions — it wastes execution time without adding security (unconstrained results are verified by the constrained caller).
- Don't use runtime-dynamic loop bounds in constrained code — the compiler must be able to unroll all loops.

## Leveraging the Type System

Noir's type system provides range guarantees that make subsequent constraints cheaper — the compiler knows what values a type can hold and can emit simpler arithmetic as a result. Use typed values instead of manual field arithmetic.

### Use `bool` Instead of Field Arithmetic

The `bool` type guarantees values are 0 or 1, so the compiler can use simpler constraints for operations on booleans. Prefer boolean operators over field multiplication for logical conditions:

```noir
// Prefer: readable, compiler knows switched[i] is 0 or 1
assert(!switched[i] | switched[i - 1]);

// Avoid: manual field encoding of the same logic
let s = switched[i] as Field;
let prev = switched[i - 1] as Field;
assert(s * (1 - prev) == 0);
```

Both compile to equivalent constraints, but the boolean version communicates intent.

## Conditionals

### Use `if/else` Expressions for Conditional Values

The compiler lowers `if cond { a } else { b }` into an optimized conditional select. Don't hand-roll the arithmetic:

```noir
// Prefer: clear intent
let val = if condition { x } else { y };

// Avoid: manual select
let c = condition as Field;
let val = c * (x - y) + y;
```

### Hoist Assertions Out of Conditional Branches

When both branches of an `if/else` contain assertions, extract the condition into a value and assert once. The compiler optimizes a single assertion against a conditional value better than separate assertions in each branch:

```noir
// Prefer: one assertion, compiler optimizes the conditional select
let expected = if condition { a } else { b };
assert_eq(result, expected);

// Avoid: duplicated assertions in each branch
if condition {
    assert_eq(result, a);
} else {
    assert_eq(result, b);
}
```

## Assertions

Use `assert_eq` over `assert(x == y)`. It provides better error messages on failure and reads more naturally:

```noir
assert_eq(result[i], expected);
```

## Comparison Costs

Integer comparisons (`<`, `<=`, `>`, `>=`) require range checks, which cost gates. Equality checks (`==`) are cheaper. Strategies to reduce comparison costs:

- **Avoid redundant comparisons**: If you need to check `i <= index` in a loop, do it once per iteration — don't check the same condition in multiple places.
- **Don't over-optimize comparisons**: Replacing a simple `<=` with flag-tracking (`if i == index { flag = true }`) adds state and may produce *more* gates. Always measure before committing to a "cleverer" approach.

## Summary Checklist

When writing or reviewing Noir code:

1. Can any in-circuit computation be replaced with hint-and-verify?
2. Are you hinting only final results, not intermediate scaffolding?
3. Are boolean conditions using `bool` types and operators, not Field arithmetic?
4. Are conditional values using `if/else` expressions, not manual selects?
5. Are assertions using `assert_eq` where applicable?
6. Are constrained and unconstrained paths optimized for their respective targets?
7. Do all loops in constrained code have comptime-known bounds?
