trait Failer {
    fn fail(self) {
        std::static_assert(false, "trait default failure");
    }
}

struct S {}

impl Failer for S {}

fn main() {
    S {}.fail();
}
