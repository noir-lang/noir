# Borrowing fields through references

`&foo.bar` and `&mut foo.bar`, where `foo` is a reference, borrow the field in place: the
result points at the field inside the referenced struct rather than at a copy of it. This
matches `&x` on a variable and `&s.x` on a local struct, which also alias what they borrow.

The elaborator implements this as an *offset* member access. The member access is elaborated
without auto-dereferencing and yields a reference to the field, and the outer `&`/`&mut` is
dropped instead of adding a second reference (`skip` on the prefix expression).

## Which reference is offset into

When the struct is behind several references (`outer: &mut &mut Foo`, or a struct field that
itself holds a `&mut Foo`), only the innermost reference, the one directly around the struct,
is offset into. Every reference around it is dereferenced first, so `&outer.x` is
`&(*outer).x` and has type `&Field`, never a reference to a reference.

## Mutability

The borrow has the mutability the user asked for, never more:

- `&foo.bar` is a `&Bar` even when `foo` is a `&mut Foo`. Inheriting `foo`'s mutability would
  turn an explicit `&` into a writable `&mut`, which can then be passed anywhere a `&mut Bar`
  is expected.
- `&mut foo.bar` requires the innermost reference to be `&mut`, and is otherwise rejected with
  "Cannot take a mutable reference to a value behind a `&` reference".

Only the innermost reference's mutability counts; outer references do not restrict it. This is
the same rule assignments follow: references are copyable values, so a `&mut` stored behind a
`&` can be copied out and written through, and `x.0.0 = 1` is accepted for
`x: &(&mut (Field,),)` while rejected for `x: &mut (&(Field,),)`.

Downstream, SSA allows a `&mut T` value to be used as `&T` (see
[ssa_reference_mutability.md](ssa_reference_mutability.md)), so typing a field borrow `&T` while
the underlying SSA value is the field's `&mut` address needs no conversion.
