#[outer_attribute]
mod foo {
    #![some_attribute]
    fn x() {}
    fn y() {}
}

contract bar {}

#[some_attribute]
mod another_module {}

#[outer_attribute_func]
mod yet_another_module {
    #![super::inner_attribute_func]
    fn foo() {}
}

#[outer_attribute_separate_module]
mod separate_module;

comptime mut global counter = 0;

fn increment_counter() {
    counter += 1;
}

fn outer_attribute_func(m: Module) {
    assert_eq(m.name(), quote { yet_another_module });
    increment_counter();
}

fn inner_attribute_func(m: Module) {
    assert_eq(m.name(), quote { yet_another_module });
    increment_counter();
}

fn outer_attribute_separate_module(m: Module) {
    assert_eq(m.name(), quote { separate_module });
    increment_counter();
}

fn main() {
    comptime
    {
        // Check Module::is_contract
        let foo = quote { foo }.as_module().unwrap();
        assert(!foo.is_contract());

        let bar = quote { bar }.as_module().unwrap();
        assert(bar.is_contract());

        let another_module = quote { another_module }.as_module().unwrap();

        // Check Module::functions
        assert_eq(foo.functions().len(), 2);
        assert_eq(bar.functions().len(), 0);

        // Check Module::name
        assert_eq(foo.name(), quote { foo });
        assert_eq(bar.name(), quote { bar });

        // Check Module::has_named_attribute
        assert(foo.has_named_attribute(quote { some_attribute }));
        assert(foo.has_named_attribute(quote { outer_attribute }));
        assert(!bar.has_named_attribute(quote { some_attribute }));
        assert(another_module.has_named_attribute(quote { some_attribute }));
    }

    assert_eq(counter, 4);
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
