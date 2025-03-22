
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
    