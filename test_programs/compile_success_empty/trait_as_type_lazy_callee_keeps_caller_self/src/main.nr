// `Outer::probe` references `Inner::hidden` before `Inner::hidden`'s body has been elaborated, so
// the compiler elaborates `Inner::hidden` in the middle of `Outer::probe`. `Self` in the rest of
// `Outer::probe` must still be `Outer`, so `Self::after()` is `Outer::after()`.
trait Marker {}

struct Inner {}
impl Marker for Inner {}

struct Outer {}

impl Outer {
    fn after() -> bool {
        true
    }

    fn probe(_self: Self) -> bool {
        let _hidden = Inner::hidden();
        Self::after()
    }
}

impl Inner {
    fn hidden() -> impl Marker {
        Inner {}
    }

    fn after() -> bool {
        false
    }
}

fn main() {
    assert(Outer::probe(Outer {}));
    assert(!Inner::after());
}
