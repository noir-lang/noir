// The compiler injects entry-point validation by resolving `std::validate::Validate` explicitly in
// the standard library, so a `Validate` of the user's own that shadows the prelude one does not
// divert it. Despite the local trait below, `vec` is still checked against the stdlib `BoundedVec`
// invariant, and `len = 5` against capacity `3` fails at runtime.
pub trait Validate {
    fn validate(self);
}

fn main(vec: BoundedVec<Field, 3>) {}
