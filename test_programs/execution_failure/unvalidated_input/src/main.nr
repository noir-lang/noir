// An entry-point input whose type does not implement `Validate` is flagged with the
// `unvalidated_input` warning (captured below). `Wrapper` does not derive `Validate`, so it isn't
// checked on entry: the inner `Bounded` carries its own `#[validate]` invariant, but nothing runs
// it automatically, so the explicit `w.inner.validate()` below has to — and it trips on the bad
// input.
//
// Adding `#[derive(Validate)]` to `Wrapper` would both silence the warning and validate the inner
// `Bounded` automatically (the derived `validate` recurses into the field), making the explicit
// call redundant.
use std::validate::validate;

#[derive(Validate)]
#[validate(Bounded::in_range)]
struct Bounded {
    value: u32,
}

impl Bounded {
    fn in_range(&self) {
        assert(self.value <= 100, "value out of range");
    }
}

struct Wrapper {
    inner: Bounded,
}

fn main(w: Wrapper) {
    w.inner.validate();
}
