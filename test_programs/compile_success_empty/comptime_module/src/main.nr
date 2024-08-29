mod foo {
    fn x() {}
    fn y() {}
}

contract bar {}

fn main() {
    comptime
    {
        // Check Module::is_contract
        let foo = quote { foo }.as_module().unwrap();
        assert(!foo.is_contract());

        let bar = quote { bar }.as_module().unwrap();
        assert(bar.is_contract());

        // Check Module::functions
        assert_eq(foo.functions().len(), 2);
        assert_eq(bar.functions().len(), 0);

        // Check Module::name
        assert_eq(foo.name(), quote { foo });
        assert_eq(bar.name(), quote { bar });
    }
}

// docs:start:as_module_example
mod baz {
    mod qux {}
}

#[test]
fn as_module_test() {
    comptime
    {
        let my_mod = quote { baz::qux }.as_module().unwrap();
        assert_eq(my_mod.name(), quote { qux });
    }
}
// docs:end:as_module_example
