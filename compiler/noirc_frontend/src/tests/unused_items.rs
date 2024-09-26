use crate::{
    hir::{def_collector::dc_crate::CompilationError, resolution::errors::ResolverError},
    tests::assert_no_errors,
};

use super::get_program_errors;

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
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "bar");
    assert_eq!(item.item_type(), "import");
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
    use foo::baz;
    use foo::Foo;

    impl Foo for Field {
    }

    fn main() {
        baz();
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "bar");
    assert_eq!(item.item_type(), "import");
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
        bar();
    }

    fn bar() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "foo");
    assert_eq!(item.item_type(), "function");
}

#[test]
fn errors_on_unused_struct() {
    let src = r#"
    struct Foo {}
    struct Bar {}

    fn main() {
        let _ = Bar {};
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "Foo");
    assert_eq!(item.item_type(), "struct");
}

#[test]
fn errors_on_unused_trait() {
    let src = r#"
    trait Foo {}
    trait Bar {}

    pub struct Baz {
    }

    impl Bar for Baz {}

    fn main() {
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "Foo");
    assert_eq!(item.item_type(), "trait");
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
    type Bar = Field;
    pub fn bar(_: Bar) {}
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::UnusedItem { ident, item }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(ident.to_string(), "Foo");
    assert_eq!(item.item_type(), "type alias");
}

#[test]
fn errors_if_type_alias_aliases_more_private_type() {
    let src = r#"
    struct Foo {}
    pub type Bar = Foo;
    pub fn no_unused_warnings(_b: Bar) {
        let _ = Foo {};
    }
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TypeIsMorePrivateThenItem {
        typ, item, ..
    }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(typ, "Foo");
    assert_eq!(item, "Bar");
}

#[test]
fn errors_if_type_alias_aliases_more_private_type_in_generic() {
    let src = r#"
    pub struct Generic<T> { value: T }
    struct Foo {}
    pub type Bar = Generic<Foo>;
    pub fn no_unused_warnings(_b: Bar) {
        let _ = Foo {};
        let _ = Generic { value: 1 };
    }
    fn main() {}
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::ResolverError(ResolverError::TypeIsMorePrivateThenItem {
        typ, item, ..
    }) = &errors[0].0
    else {
        panic!("Expected an unused item error");
    };

    assert_eq!(typ, "Foo");
    assert_eq!(item, "Bar");
}
