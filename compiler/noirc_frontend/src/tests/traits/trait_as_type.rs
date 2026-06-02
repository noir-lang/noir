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
