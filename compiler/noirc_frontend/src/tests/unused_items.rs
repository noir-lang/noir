use noirc_errors::CustomDiagnostic;

use crate::hir::def_collector::dc_crate::CompilationError;
use crate::parser::ParserErrorReason;
use crate::test_utils::{GetProgramOptions, get_program_with_options};
use crate::tests::{assert_no_errors, check_errors, get_program_errors};

#[test]
fn errors_on_unused_private_import() {
    let src = r#"
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    use foo::bar;
             ^^^ unused import bar
             ~~~ unused import
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_pub_crate_import() {
    let src = r#"
    mod foo {
        pub fn bar() {}
        pub fn baz() {}

        pub trait Foo {
        }
    }

    pub(crate) use foo::bar;
                        ^^^ unused import bar
                        ~~~ unused import
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;
    check_errors(src);
}

#[test]
fn unused_global_clashing_with_function_is_reported_as_global() {
    // The global and the function share the `values` namespace, so collecting the
    // function clashes with the already-collected global. The unused warning must
    // still describe the surviving item (the global) and point at its location,
    // rather than borrowing the function's kind. The duplicate-definition secondaries
    // stay kind-neutral ("First/Second definition found here") so they don't mislabel
    // the global as a function the way the primary's `typ` does.
    let src = r#"
    global N: u32 = 10;
           ^ unused global N
           ~ unused global
           ~ First definition found here
    fn N() {}
       ^ Duplicate definitions of function with name N found
       ~ Second definition found here
    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn cross_namespace_name_clash_still_warns_on_unused_value() {
    // A type-namespace item (`struct N`) and a value-namespace item (`fn N`) may legally
    // share a name within a module, so this is *not* a duplicate-definition error and the
    // program compiles. The usage tracker keys unused items by `(namespace, name)`, so the
    // struct and the function occupy separate slots: constructing the struct clears only the
    // type-namespace entry, leaving the genuinely unused `fn N` to warn.
    let src = r#"
    struct N {}
    fn N() {}
       ^ unused function N
       ~ unused function
    fn main() {
        let _ = N {};
    }
    "#;
    check_errors(src);
}

#[test]
fn unused_item_warns_at_definition_not_at_shadowing_use_site() {
    // A value-namespace generic (`let N`) shadows an unused value-namespace global of the same
    // name. Resolving the generic in `0..N` runs a speculative path-resolution probe that touches
    // the global's unused entry and then rolls back. The rollback must restore that entry under its
    // *original* key (the global's definition location), not the probe's use-site ident, so the
    // surviving warning still points at the `global N` definition rather than the `0..N` use.
    let src = r#"
    global N: u32 = 10;
           ^ unused global N
           ~ unused global
    fn count<let N: u32>() -> u32 {
        let mut sum = 0;
        for _ in 0..N {
            sum += 1;
        }
        sum
    }
    fn main() {
        let _ = count::<3>();
    }
    "#;
    check_errors(src);
}

#[test]
fn cross_namespace_name_clash_still_warns_on_unconstructed_type() {
    // Mirror of the case above: calling `N()` clears only the value-namespace entry, leaving the
    // never-constructed `struct N` to warn. Elaborating the call speculatively resolves `N` as a
    // `Type::method` receiver, but that probe only *references* the type (it doesn't mark it
    // used), so the struct is still reported as never constructed.
    let src = r#"
    struct N {}
           ^ struct `N` is never constructed
           ~ struct is never constructed
    fn N() {}
    fn main() {
        N();
    }
    "#;
    check_errors(src);
}

#[test]
fn unused_dual_namespace_import_warns_once() {
    // `use foo::N` brings both the type-namespace `struct N` and the value-namespace `fn N` into
    // scope, but a `use` is a single syntactic unit, so an unused import warns exactly once (not
    // once per namespace).
    let src = r#"
    mod foo {
        pub struct N {}
        pub fn N() {}
    }

    use foo::N;
             ^ unused import N
             ~ unused import

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn unused_value_namespace_import_not_cleared_by_type_namespace_use() {
    // `use foo::N` only imports the value-namespace `fn N`. Constructing the local type-namespace
    // `struct N` uses that struct, not the import, so the import is still unused and must warn.
    // TODO(#13008) TODO(#13009): Arguably the `pub fn N` itself is unused, but at the moment
    // a) `pub` items are not tracked at all and b) the import marks functions as used.
    let src = r#"
    mod foo {
        pub fn N() {}
    }

    use foo::N;
             ^ unused import N
             ~ unused import

    struct N {}

    fn main() {
        let _ = N {};
    }
    "#;
    check_errors(src);
}

#[test]
fn unused_separate_namespace_imports_warn_independently() {
    // `use foo::N` and `use bar::N` are two distinct `use` statements that happen to import the
    // same name into different namespaces (the type-namespace `struct N` and the value-namespace
    // `fn N`). Each is its own syntactic unit, so using `N` only as a type uses `foo::N`; the
    // unused `bar::N` must still warn, even though both share the name `N`.
    let src = r#"
    mod foo { pub struct N {} }
    mod bar { pub fn N() {} }

    use foo::N;
    use bar::N;
             ^ unused import N
             ~ unused import

    fn main() {
        let _: N = N {};
    }
    "#;
    check_errors(src);
}

#[test]
fn unused_type_import_not_masked_by_value_call() {
    // `use foo::N` imports the type-namespace `struct N`; `use bar::N` imports the value-namespace
    // `fn N`. Calling `N()` uses only the value import. Resolving the call speculatively probes `N`
    // as a type (checking for `Type::method` syntax), but that probe is rolled back when it turns
    // out `N` isn't a trait static method, so it doesn't mark the type import used — the
    // never-referenced `use foo::N` still warns.
    let src = r#"
    mod foo { pub struct N {} }
    mod bar { pub fn N() {} }

    use foo::N;
             ^ unused import N
             ~ unused import
    use bar::N;

    fn main() {
        N();
    }
    "#;
    check_errors(src);
}

#[test]
fn trait_static_method_call_marks_trait_import_used() {
    // A genuine `T::make(..)` trait static method call (with `Self` inferable here from the `let`
    // type) is resolved *only* by the speculative trait-static-method probe. When the probe
    // succeeds its usage marks are committed, so the trait import `T` must not be reported unused.
    let src = r#"
    mod foo {
        pub trait T { fn make() -> Self; }
        pub struct S {}
        impl T for S { fn make() -> Self { S {} } }
    }

    use foo::T;
    use foo::S;

    fn main() {
        let _: S = T::make();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn import_used_only_in_signature_not_dropped_by_speculative_probe() {
    // `Bar` is imported into `mod m` and used *only* in `mk`'s parameter type. Calling `crate::m::mk`
    // via a multi-segment path from a global initializer (where `mk`'s meta is still unresolved)
    // routes through the speculative trait-static-method probe. The probe resolves `mk`'s meta —
    // marking `Bar` used — then fails (mk is not a trait method). Resolving a function's meta is a
    // committed structural change, so its usage marks must survive the probe's rollback; otherwise
    // `mk` is never re-resolved and `Bar` is wrongly reported unused.
    let src = r#"
    mod other { pub struct Bar {} }
    mod m {
        use crate::other::Bar;
        pub fn mk(_x: Bar) -> u32 { 0 }
    }
    global X: u32 = crate::m::mk(crate::other::Bar {});
    fn main() { let _ = X; }
    "#;
    assert_no_errors(src);
}

#[test]
fn inherent_assoc_fn_return_import_used_from_global() {
    // `Bar` is imported into `mod m` and used only in the return type of the inherent associated
    // function `Foo::new`, invoked from the global initializer `X`. Resolving `Foo::new` from the
    // global routes through the speculative trait-static-method probe; its import must not be
    // dropped by the probe's rollback.
    let src = r#"
    mod other {
        pub struct Bar {}
        pub fn mk() -> Bar { Bar {} }
    }
    mod m {
        use crate::other::Bar;
        use crate::other::mk;
        pub struct Foo {}
        impl Foo {
            pub fn new() -> Bar { mk() }
        }
    }
    global X: crate::other::Bar = crate::m::Foo::new();
    fn main() { let _ = X; }
    "#;
    assert_no_errors(src);
}

#[test]
fn same_namespace_imports_with_same_name_clash() {
    // Contrast with the cross-namespace case above: when two `use`s bring the same name into the
    // *same* namespace (here both `foo::N` and `bar::N` are type-namespace `struct`s), they are no
    // longer distinct slots — the second import collides with the first, which is a duplicate-import
    // error rather than a pair of independently-tracked unused imports.
    let src = r#"
    mod foo { pub struct N {} }
    mod bar { pub struct N {} }

    use foo::N;
             ~ First definition found here
    use bar::N;
             ^ Duplicate definitions of import with name N found
             ~ Second definition found here

    fn main() {
        let _: N = N {};
    }
    "#;
    check_errors(src);
}

#[test]
fn repeated_import_of_same_name_does_not_warn_when_used() {
    // Two identical `use`s are a duplicate-import error, but each is still tracked as its own entry
    // (keyed by location). Using `N` must clear *every* same-name entry in that namespace, so no
    // spurious unused-import warning survives alongside the duplicate-definition error.
    let src = r#"
    mod foo { pub struct N {} }

    use foo::N;
             ~ First definition found here
    use foo::N;
             ^ Duplicate definitions of import with name N found
             ~ Second definition found here

    fn main() {
        let _: N = N {};
    }
    "#;
    check_errors(src);
}

#[test]
fn repeated_unused_import_warns_on_each_use() {
    // Mirror of the case above when the name is never used: the duplicate-definition error still
    // fires, and because each `use` is its own location-keyed entry, each unused line warns
    // independently (two warnings, not one).
    let src = r#"
    mod foo { pub struct N {} }

    use foo::N;
             ^ unused import N
             ~ unused import
             ~ First definition found here
    use foo::N;
             ^ unused import N
             ^ Duplicate definitions of import with name N found
             ~ unused import
             ~ Second definition found here

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_function() {
    let src = r#"
    contract some_contract {
        // This function is unused, but it's a contract entrypoint
        // so it should not produce a warning
        fn foo() -> pub Field {
            1
        }
    }


    fn foo() {
       ^^^ unused function foo
       ~~~ unused function
        bar();
    }

    fn bar() {}
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_struct() {
    let src = r#"
    struct Foo {}
           ^^^ struct `Foo` is never constructed
           ~~~ struct is never constructed
    struct Bar {}

    fn main() {
        let _ = Bar {};
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unused_trait() {
    let src = r#"
    trait Foo {}
          ^^^ unused trait Foo
          ~~~ unused trait
    trait Bar {}

    pub struct Baz {
    }

    impl Bar for Baz {}

    fn main() {
    }
    "#;
    check_errors(src);
}

#[test]
fn silences_unused_variable_warning() {
    let src = r#"
    fn main() {
        #[allow(unused_variables)]
        let x = 1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_on_unused_type_alias() {
    let src = r#"
    type Foo = Field;
         ^^^ unused type alias Foo
         ~~~ unused type alias
    type Bar = Field;
    pub fn bar(_: Bar) {}
    "#;
    check_errors(src);
}

#[test]
fn warns_on_unused_global() {
    let src = r#"
    global foo: u32 = 1;
           ^^^ unused global foo
           ~~~ unused global
    global bar: Field = 1;

    fn main() {
        let _ = bar;
    }
    "#;
    check_errors(src);
}

#[test]
fn does_not_warn_on_unused_global_if_it_has_an_abi_attribute() {
    let src = r#"
    contract foo {
        #[abi(notes)]
        global bar: u64 = 1;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    contract moo {
        #[abi(dummy)]
        struct Foo { bar: u8 }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_warn_on_unused_function_if_it_has_an_export_attribute() {
    let src = r#"
    #[export]
    fn foo() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_inner_struct_when_parent_is_used() {
    let src = r#"
    struct Bar {
        inner: [Field; 3],
    }

    struct Foo {
        a: Field,
        bar: Bar,
    }

    fn main(foos: [Foo; 1]) {
        assert_eq(foos[0].a, 10);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    contract moo {
        #[abi(functions)]
        struct Foo {
            a: Field,
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_indirect_struct_if_it_has_an_abi_attribute() {
    let src = r#"
    contract moo {
        struct Bar {
            field: Field,
        }

        #[abi(functions)]
        struct Foo {
            bar: Bar,
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn no_warning_on_self_in_trait_impl() {
    let src = r#"
    struct Bar {}

    trait Foo {
        fn foo(self, a: u64);
    }

    impl Foo for Bar {
        fn foo(self, _a: u64) {}
    }

    fn main() {
        let _ = Bar {};
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolves_trait_where_clause_in_the_correct_module() {
    // This is a regression test for https://github.com/noir-lang/noir/issues/6479
    let src = r#"
    mod foo {
        pub trait Foo {}
    }

    use foo::Foo;

    pub trait Bar<T>
    where
        T: Foo,
    {}
    "#;
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_impl_method_is_called() {
    let src = "
    struct Bar {}

    impl Bar {
        fn foo() {}
    }

    pub fn main() {
        Bar::foo()
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_trait_method_is_called() {
    let src = "
    struct Bar {}

    pub trait Foo {
        fn foo();
    }

    impl Foo for Bar {
        fn foo() {}
    }

    pub fn main() {
        Bar::foo()
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_mentioned_in_let_type() {
    let src = "
    struct Bar {}

    fn foo(array: [Bar; 1]) {
        let _: Bar = array[0];
    }

    fn main() {
        let _ = foo;
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_mentioned_in_return_type() {
    let src = "
    struct Bar {}

    fn foo(array: [Bar; 1]) -> Bar {
        array[0]
    }

    fn main() {
        let _ = foo;
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_passed_in_generic_args_in_constructor() {
    let src = "
    struct Bar {}

    struct Generic<T> {}

    fn main() {
        let _ = Generic::<Bar> {};
    }
    ";
    assert_no_errors(src);
}

#[test]
fn considers_struct_as_constructed_if_passed_in_generic_args_in_function_call() {
    let src = "
    struct Bar {}

    fn foo<T>() {}

    fn main() {
        let _ = foo::<Bar>();
    }
    ";
    assert_no_errors(src);
}

#[test]
fn does_not_consider_struct_as_constructed_if_mentioned_in_function_argument() {
    let src = "
    struct Bar {}
           ^^^ struct `Bar` is never constructed
           ~~~ struct is never constructed

    fn foo(_: [Bar; 1]) {}

    fn main() {
        foo();
        ^^^^^ Function expects 1 parameter but 0 were given
    }
    ";
    check_errors(src);
}

#[test]
fn allow_dead_code_on_unused_function() {
    let src = "
    #[allow(dead_code)]
    fn foo() {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_struct() {
    let src = "
    #[allow(dead_code)]
    struct Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_trait() {
    let src = "
    #[allow(dead_code)]
    trait Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn allow_dead_code_on_unused_enum() {
    let src = "
    #[allow(dead_code)]
    enum Foo {}

    fn main() {
    }
    ";
    assert_no_errors(src);
}

#[test]
fn errors_on_unused_impl_function() {
    let src = "
    pub struct Foo {}

    impl Foo {
        fn foo() {}
           ^^^ unused function foo
           ~~~ unused function
    }

    fn main() {}
    ";
    check_errors(src);
}

#[test]
fn does_not_error_on_unused_impl_function() {
    let src = "
    pub struct Foo {}

    impl Foo {
        fn foo() {}
    }

    fn main() {
        let _ = Foo::foo();
    }
    ";
    assert_no_errors(src);
}

#[test]
fn does_not_error_on_used_impl_method() {
    let src = "
    pub struct Foo {}

    impl Foo {
        fn foo(self) {
            let _ = self;
        }
    }

    fn main() {
        Foo {}.foo();
    }
    ";
    assert_no_errors(src);
}

#[test]
fn does_not_error_on_unused_impl_method_if_marked_as_allow_dead_code() {
    let src = "
    pub struct Foo {}

    impl Foo {
        #[allow(dead_code)]
        fn foo(self) {
            let _ = self;
        }
    }

    fn main() {}
    ";
    assert_no_errors(src);
}

fn unknown_lint_reasons(src: &str) -> Vec<String> {
    get_program_errors(src)
        .iter()
        .filter_map(|error| match error {
            CompilationError::ParseError(error) => match error.reason() {
                Some(ParserErrorReason::UnknownLint { name }) => Some(name.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn warns_on_unknown_lint_in_allow_attribute() {
    // `dead_cod` is a typo for `dead_code`, so the `allow` names a lint that
    // doesn't exist. `foo` is used, so the only diagnostic is the unknown-lint warning.
    let src = r#"
    #[allow(dead_cod)]
    fn foo() {}

    fn main() {
        foo();
    }
    "#;
    assert_eq!(unknown_lint_reasons(src), vec!["dead_cod".to_string()]);
}

#[test]
fn does_not_warn_on_known_lint_in_allow_attribute() {
    let src = r#"
    #[allow(dead_code)]
    fn foo() {}

    fn main() {}
    "#;
    assert!(unknown_lint_reasons(src).is_empty());
}

#[test]
fn typo_in_allow_does_not_suppress_the_lint() {
    // A misspelled slug must not silence the warning it looks like it targets. `x` is
    // unused, so both the unknown-lint warning and the unused-variable warning fire.
    // `allow_parser_errors` lets elaboration run past the unknown-lint parse warning,
    // mirroring the real compiler, which never blocks elaboration on parser warnings.
    let src = r#"
    fn main() {
        #[allow(unused_variabl)]
        let x = 1;
    }
    "#;
    let options = GetProgramOptions { allow_parser_errors: true, ..Default::default() };
    let errors = get_program_with_options(src, options).2;

    let unknown_lints: Vec<_> = errors
        .iter()
        .filter_map(|error| match error {
            CompilationError::ParseError(error) => match error.reason() {
                Some(ParserErrorReason::UnknownLint { name }) => Some(name.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(unknown_lints, vec!["unused_variabl".to_string()]);

    let has_unused_variable_warning = errors
        .iter()
        .any(|error| CustomDiagnostic::from(error).message.contains("unused variable"));
    assert!(has_unused_variable_warning, "expected unused-variable warning, got {errors:#?}");
}
