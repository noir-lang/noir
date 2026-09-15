//! Functions returning `impl Trait`.
use crate::{elaborator::UnstableFeature, tests::check_errors_using_features};

/// Show that `impl Trait` functions mutually calling each other do not compile currently.
/// The example below does compile in Rust, but we need to refactor how types are substituted
/// to make it work in the elaborator.
#[test]
fn mutually_recursive_impl_trait_functions() {
    let src = r#"
    trait Foo {}

    struct Bar {}
    struct Baz {}
    impl Foo for Bar {}
    impl Foo for Baz {}

    fn main() {
        let _bar = bar(true);
        let _baz = baz(true);
    }

    fn bar(recur: bool) -> impl Foo {
        if recur {
            let _baz = baz(false);
        }
        Bar {}
    }

    fn baz(recur: bool) -> impl Foo {
        if recur {
            let _bar = bar(false);
                       ^^^ Dependency cycle found
                       ~~~ 'bar' recursively depends on itself: 'impl Trait' could not be resolved to the type of the function body
        }
        Baz {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

/// Show that the elaborator handle acyclic `impl Trait` functions that appear
/// out of dependency order, by elaborating the callee on the fly.
#[test]
fn out_of_order_impl_trait_functions() {
    let src = r#"
    trait Foo {}

    struct Bar {}
    struct Baz {}
    impl Foo for Bar {}
    impl Foo for Baz {}

    fn main() {
        let _bar = bar();
    }

    fn bar() -> impl Foo {
        let _baz = baz();
        Bar {}
    }

    fn baz() -> impl Foo {
        Baz {}
    }
    "#;
    // Not using `assert_no_errors` because it does not enable the feature.
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

// The tests below reference an `impl Trait` function before its body has been elaborated, so the
// elaborator elaborates it in the middle of the caller's body. Each one checks that a different
// piece of the caller's context is the same after that as before it.

#[test]
fn lazily_elaborated_impl_trait_callee_keeps_callers_self_type() {
    let src = r#"
    trait Marker {}

    struct Inner {}
    impl Marker for Inner {}

    struct Outer {}

    impl Outer {
        fn after() -> bool { true }

        fn probe(_self: Self) -> bool {
            let _hidden = Inner::hidden();
            Self::after()
        }
    }

    impl Inner {
        fn hidden() -> impl Marker { Inner {} }
        fn after() -> Field { 1 }
    }

    fn main() {
        let _ok: bool = Outer::probe(Outer {});
        let _field: Field = Inner::after();
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_keeps_callers_trait_context() {
    let src = r#"
    trait Marker {}

    struct Inner {}
    impl Marker for Inner {}

    struct Outer {}

    trait Probe {
        fn required() -> bool;

        fn probe() -> bool {
            let _hidden = Inner::hidden();
            Self::required()
        }
    }

    impl Probe for Outer {
        fn required() -> bool { true }
    }

    impl Inner {
        fn hidden() -> impl Marker { Inner {} }
    }

    fn main() {
        let _ok = Outer::probe();
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_keeps_callers_generics() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    fn main() {
        let _x = caller(1);
    }

    fn caller<T>(x: T) -> T {
        let _hidden = hidden();
        let y: T = x;
        y
    }

    fn hidden() -> impl Marker { Bar {} }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_keeps_callers_trait_bounds() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    trait HasAssoc {
        type Assoc;
        fn make() -> Self::Assoc;
    }

    struct Foo {}
    impl HasAssoc for Foo {
        type Assoc = Field;
        fn make() -> Field { 1 }
    }

    fn main() {
        let _x = caller::<Foo>();
    }

    fn caller<T: HasAssoc>() -> T::Assoc {
        let _hidden = hidden();
        let x: T::Assoc = T::make();
        x
    }

    fn hidden() -> impl Marker { Bar {} }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_does_not_inherit_callers_unsafe_block() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    unconstrained fn uc() -> Field { 1 }

    fn main() {
        // Safety: test
        let _x = unsafe {
            let _hidden = hidden();
            uc()
        };
    }

    fn hidden() -> impl Marker {
        let _y = uc();
                 ^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        Bar {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_does_not_inherit_callers_unconstrained_lambda() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    unconstrained fn uc() -> Field { 1 }

    unconstrained fn run(f: fn() -> ()) {
        f();
    }

    unconstrained fn main() {
        run(|| {
            let _hidden = hidden();
        });
    }

    fn hidden() -> impl Marker {
        let _y = uc();
                 ^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        Bar {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_does_not_inherit_callers_loop() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    unconstrained fn main() {
        for _ in 0..1 {
            let _hidden = hidden(true);
        }
    }

    unconstrained fn hidden(stop: bool) -> impl Marker {
        if stop {
            break;
            ^^^^^^ break is only allowed within loops
        }
        Bar {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_does_not_inherit_callers_comptime_block() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    fn main() {
        comptime {
            let _hidden = hidden();
        }
    }

    fn hidden() -> impl Marker {
        let _q = quote { 1 };
                 ^^^^^^^^^^^ `quote` cannot be used in runtime code
                 ~~~~~~~~~~~ Wrap this in a `comptime` block or function to use it
        Bar {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}

#[test]
fn lazily_elaborated_impl_trait_callee_does_not_inherit_callers_unconstrained_args() {
    let src = r#"
    trait Marker {}

    struct Bar {}
    impl Marker for Bar {}

    unconstrained fn uc() -> Field { 1 }

    unconstrained fn run<T>(_x: T) {}

    unconstrained fn main() {
        // `hidden` is elaborated on demand while the arguments of a call to an unconstrained
        // function are being elaborated.
        run(hidden());
    }

    fn hidden() -> impl Marker {
        let f: fn() -> Field = || uc();
                                  ^^^^ Call to unconstrained function from constrained function is unsafe and must be in an unconstrained function or unsafe block
        let _ = f;
        Bar {}
    }
    "#;
    check_errors_using_features(src, &[UnstableFeature::TraitAsType]);
}
