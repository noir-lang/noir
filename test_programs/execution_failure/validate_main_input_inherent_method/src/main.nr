// An inherent `validate` method must not shadow the entry-point validation. That validation is
// resolved as the stdlib `std::validate::Validate` trait and emitted as
// `std::validate::Validate::validate(&p)`, not as `p.validate()` — a method call would prefer the
// inherent method below over the trait implementation. The inherent method is a no-op, while the
// trait implementation enforces the invariant, so `value = 200` fails on entry. The call stack
// names `<impl Validate for Percent>::validate`, confirming the trait method ran.
struct Percent {
    value: u8,
}

impl Percent {
    // A lax inherent check that accepts the out-of-range input. It is never called — the warning
    // that it is unused is itself evidence that the entry-point validation reaches for the trait
    // method below instead.
    fn validate(&self) {
        assert(self.value <= 200);
    }
}

impl std::validate::Validate for Percent {
    fn validate(&self) {
        assert(self.value <= 100, "percent out of range");
    }
}

fn main(p: Percent) {}
