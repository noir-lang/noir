// `Foo` has both an inherent `bar` and a trait `Bar::bar`. The inherent method is private to
// the `impls` module, so it is not accessible from the root. The trait `Bar` is public and in
// scope here, so `(Foo {}).bar()` must resolve to the trait method (returning 2) rather than the
// inaccessible inherent method (returning 1).
pub struct Foo {}

trait Bar {
    fn bar(self) -> u32;
}

mod impls {
    use super::{Bar, Foo};

    impl Foo {
        // Private to this module: callable here, but not from the root.
        fn bar(self) -> u32 {
            let _ = self;
            1
        }
    }

    impl Bar for Foo {
        fn bar(self) -> u32 {
            let _ = self;
            2
        }
    }

    // Exercises the inherent method from within `impls`, where it is visible. The inherent
    // method wins here (returning 1) because it is accessible from this module.
    pub fn calls_inherent_bar(foo: Foo) -> u32 {
        foo.bar()
    }
}

fn main() {
    // Resolves to the public trait method, not the private inherent one.
    assert_eq((Foo {}).bar(), 2);

    // The fully-qualified trait call agrees.
    assert_eq(<Foo as Bar>::bar(Foo {}), 2);

    // From inside `impls` the inherent method is still selected.
    assert_eq(impls::calls_inherent_bar(Foo {}), 1);
}
