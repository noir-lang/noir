# Reference mutability in SSA

SSA reference types carry mutability (`&T` vs `&mut T`), and the distinction is
**meaningful and directional**: passes may rely on it for soundness decisions.

## The rule

A value of type `A` may be used where type `B` is expected when `A` is at least
as capable as `B`:

- `&mut T` may be used as `&T` (weakening). This mirrors the frontend's only
  reference coercion, `&mut T → &T`.
- `&T` may never be used as `&mut T`: that would grant a write capability the
  value's type says it does not have.
- A **mutable** reference is **invariant** in its pointee: `&mut &mut T` is not
  usable as `&mut &T`. Otherwise a plain `&T` could be stored through the weak
  view of the cell and loaded back as `&mut T` through an alias, laundering an
  immutable reference into a writable one.
- An **immutable** reference is covariant in its pointee, and arrays/vectors
  are covariant in their element types (they are semantically immutable values;
  `array_set` produces a new array).

`Type::can_be_used_as` implements this. The SSA validator enforces it at every
typed boundary, and additionally requires `store` addresses to be `&mut`-typed.

The boundaries are every point at which a reference-typed value is produced or
consumed under a type that is written on the instruction rather than derived
from its operands. Missing any one of them reopens the whole guarantee, because
a single strengthening step is enough to launder a `&T` into a `&mut T`:

- **call arguments** and **call returns**;
- **jump arguments** against the destination block's parameters;
- **`MakeArray`** elements against the array's element types;
- **`store`** values, and **`load`** results, against the tracked allocation
  type — *and*, since that tracking only covers addresses that are an
  `allocate` result in the same function, every `load` result against its
  address's own type, which is what catches an opaque address such as a
  parameter;
- **`array_get`** results and **`array_set`** values against the array's element
  type. `MakeArray` covariance means an array is a legal home for a weakened
  reference, so without the get side the array is a laundering channel. (An
  `array_set`'s result type is its array operand's type by construction, so it
  needs no separate check.)
- **`IfElse`** operands against the result type. The result type is derived from
  the `then_value` alone, so the `else_value` is otherwise unchecked.

## What this buys

Together the rules guarantee: **in validated SSA, no write can ever happen
through a value whose type contains only immutable references.** There is no
strengthening conversion at any boundary, no store through an immutable
address, and no way to launder mutability through memory.

Note that `--validate-between-passes` is off by default, so this is a property
of SSA that has been validated, not one the release pipeline re-checks after
every pass. A pass that violates it turns into a silent miscompile rather than a
panic, which is the cost of letting other passes trust reference mutability.

Passes may therefore trust reference mutability. Load Store Forwarding uses the
*callee's formal parameter types* to decide which call arguments a callee may
write through: an argument whose formal type has no mutable reference cannot
invalidate cached loads. Before mutability was directional, a pass trusting
either side's reference types could be defeated by validator-accepted SSA that
passed a `&Field`-typed value to a `&mut Field` formal which stored through it
(noir-lang/noir-claude#1512).

## Where mutability is still erased (deliberately)

- **Alias analysis** groups types with `Type::canonicalize`/`canonical_eq`
  (mutability-insensitive). This is conservative: treating `&T` and `&mut T` as
  the same type can only report *more* aliasing, never less.

## Consequences for SSA generation

- SSA-gen materializes every borrow — including `&x` over an immutable value —
  as an `allocate -> &mut T` plus an initializing store, because stores are
  only valid through mutable reference types. The resulting value weakens to
  `&T` at its uses. The same applies to the ACIR→Brillig boundary wrappers
  that re-materialize reference arguments.
- Defunctionalization preserves exact signatures. Dispatch sites are grouped by
  their exact call-site signature and variants are matched directionally
  (`dispatch_compatible`): every dispatch argument type must be usable as the
  target's formal type, and every target return type usable as the type the
  dispatch site expects. A target function may serve several apply functions.
  (Previously signatures were canonicalized to all-immutable, which made apply
  functions forward `&Field`-typed parameters to `&mut Field` formals — the
  exact shape the validator now rejects.)

## Known residual

`Type::Function` is opaque (it carries no signature), so the validator cannot
check that a function *value* only flows to dispatch sites its signature is
compatible with. Frontend-generated SSA satisfies this because frontend
function types are invariant in parameter mutability; hand-written SSA that
violates it dispatches to the apply function's final id constraint and traps.
Making function values carry their signature would close this and allow
validating indirect calls.
