// Entry-point inputs are validated automatically by resolving `std::validate::Validate` explicitly
// in the standard library, so a user's own `Validate` that shadows the prelude one cannot divert
// it: despite the local trait below, `vec` is checked against the stdlib `BoundedVec` invariant and
// fails here (`len = 5` exceeds capacity `3`). The synthesized call is elaborated without marking
// `vec` used, so the unused-variable warning still fires — the captured output shows both.
pub trait Validate {
    fn validate(self);
}

fn main(vec: BoundedVec<Field, 3>) {}
