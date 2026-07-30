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
typed boundary — call arguments, call returns, jump arguments, `MakeArray`
elements, and load/store against the tracked allocation type — and additionally
requires `store` addresses to be `&mut`-typed.

## What this buys

Together the rules guarantee: **in validated SSA, no write can ever happen
through a value whose type contains only immutable references.** There is no
strengthening conversion at any boundary, no store through an immutable
address, and no way to launder mutability through memory.

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
