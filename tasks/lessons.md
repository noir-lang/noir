# Lessons Learned

## Don't trust a bug report's stated invariant — verify it against current code

**Failure mode:** Issue #12997 (item 3) asserted that comptime auto-deref pointers are "always
mutable" and recommended tightening the match `Value::Pointer(element, true, _)` to
`(element, true, true)`. Implementing that literally broke auto-deref of *immutable* bindings:
indexing an immutable array yields `Pointer(.., /*auto_deref*/ true, /*mutable*/ false)`, which
then stopped being dereferenced.

**Detection signal:** A regression test (`arr[0].0` on an immutable array) returned the raw
`Pointer(.., true, false)` instead of the dereferenced value — caught immediately, before the
change shipped.

**Prevention rule:** Audit issues often predate code changes (here, `SignedField` removal and
pointer-mutability semantics). Before encoding a claimed invariant into a match pattern, confirm
it with a test that exercises the boundary (mutable *and* immutable). When the literal
recommendation contradicts reality, document the *actual* invariant in code and note the deviation
in the PR rather than following the suggestion blindly.

## A defensive refactor with no observable behavior change needs a unit test, not an integration test

**Failure mode:** Item 6 (deep-copy in `move_struct`) was framed as a behavioral fix, so the first
regression tests were comptime programs moving an array/struct and checking for aliasing. They
passed *with and without* the fix — comptime array index-assignment already copies (arrays hold
`im::Vector<Value>`, not `Shared` cells), so the array path never aliased observably.

**Detection signal:** Reverting the fix left the "regression" tests green.

**Prevention rule:** When a change is purely defensive (breaks sharing one level deeper than any
reachable path exercises), test the unit directly. Here, a direct `move_struct` unit test on a
`Value::Array` holding a `Value::Tuple` (whose `Shared` cell survives a shallow `Value` clone)
goes red without the fix and green with it. Always confirm red→green by reverting the
implementation, not just by watching the test pass.
