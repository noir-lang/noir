// `#[validate(Type::method)]` lets a type assert an invariant of its own on top of the field
// recursion `#[derive(Validate)]` generates. Here the invariant (`lo <= hi`) spans two fields, so
// the field recursion alone could never check it; the derived `validate` calls `Bounds::ordered`
// before descending into the fields.
use std::validate::validate;

#[derive(Validate)]
#[validate(Bounds::ordered)]
struct Bounds {
    lo: u32,
    hi: u32,
}

impl Bounds {
    fn ordered(&self) {
        assert(self.lo <= self.hi, "Bounds: lo must not exceed hi");
    }
}

fn main(lo: u32, hi: u32) {
    let bounds = Bounds { lo, hi };
    bounds.validate();
}
