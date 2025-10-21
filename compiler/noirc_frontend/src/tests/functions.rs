use crate::tests::{assert_no_errors, check_errors};

#[test]
fn resolve_empty_function() {
    let src = "
        fn main() {

        }
    ";
    assert_no_errors(src);
}

#[test]
fn resolve_basic_function() {
    let src = r#"
        fn main(x : Field) {
            let y = x + x;
            assert(y == x);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn resolve_call_expr() {
    let src = r#"
        fn main(x : Field) {
            let _z = foo(x);
        }

        fn foo(x : Field) -> Field {
            x
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn unconditional_recursion_fail() {
    // These examples are self recursive top level functions, which would actually
    // not be inlined in the SSA (there is nothing to inline into but self), so it
    // wouldn't panic due to infinite recursion, but the errors asserted here
    // come from the compilation checks, which does static analysis to catch the
    // problem before it even has a chance to cause a panic.
    let sources = vec![
        r#"
        fn main() {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main()
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            if main() { true } else { false }
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            if true { main() } else { main() }
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main() + main()
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            1 + main()
        }
        "#,
        r#"
        fn main() -> pub bool {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let _ = main();
            true
        }
        "#,
        r#"
        fn main(a: u64, b: u64) -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            main(a + b, main(a, b))
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            foo(1, main())
        }
        fn foo(a: u64, b: u64) -> u64 {
            a + b
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let (a, b) = (main(), main());
            a + b
        }
        "#,
        r#"
        fn main() -> pub u64 {
           ^^^^ function `main` cannot return without recursing
           ~~~~ function cannot return without recursing
            let mut sum = 0;
            for i in 0 .. main() {
                sum += i;
            }
            sum
        }
        "#,
    ];

    for src in sources {
        check_errors(src);
    }
}

#[test]
fn unconditional_recursion_pass() {
    let sources = vec![
        r#"
        fn main() {
            if false { main(); }
        }
        "#,
        r#"
        fn main(i: u64) -> pub u64 {
            if i == 0 { 0 } else { i + main(i-1) }
        }
        "#,
        // Only immediate self-recursion is detected.
        r#"
        fn main() {
            foo();
        }
        fn foo() {
            bar();
        }
        fn bar() {
            foo();
        }
        "#,
        // For loop bodies are not checked.
        r#"
        fn main() -> pub u64 {
            let mut sum = 0;
            for _ in 0 .. 10 {
                sum += main();
            }
            sum
        }
        "#,
        // Lambda bodies are not checked.
        r#"
        fn main() {
            let foo = || main();
            foo();
        }
        "#,
    ];

    for src in sources {
        assert_no_errors(src);
    }
}

#[test]
fn allows_multiple_underscore_parameters() {
    let src = r#"
        pub fn foo(_: i32, _: i64) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn cannot_return_slice_from_main() {
    let src = r#"
    fn main() -> pub [Field]{
       ^^^^ Invalid type found in the entry point to a program
       ~~~~ Slice is not a valid entry point type. Found: [Field]
        &[1,2]
        
    }
        "#;
    check_errors(src);
}
