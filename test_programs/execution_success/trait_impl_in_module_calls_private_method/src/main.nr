// A trait impl whose method body calls a module-private inherent method. Both impls are declared
// inside `impls`, so the inherent `secret` is visible to the trait method there. `nargo expand`
// must emit both impls inside `impls` (not hoisted to the root); otherwise the trait method body
// would lose access to the private `secret` and the expanded program would not compile.
pub struct Foo {}

trait Bar {
    fn bar(self) -> u32;
}

mod impls {
    use super::{Bar, Foo};

    impl Foo {
        fn secret(self) -> u32 {
            let _ = self;
            42
        }
    }

    impl Bar for Foo {
        fn bar(self) -> u32 {
            self.secret()
        }
    }
}

fn main() {
    assert_eq((Foo {}).bar(), 42);
}
