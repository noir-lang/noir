#[outer_attribute]
mod foo {
    #![some_attribute]
    pub fn x() {}
    pub fn y() {}

    struct Struct1 {}
}

contract bar {}

#[some_attribute]
mod another_module {}

#[outer_attribute_func]
mod yet_another_module {
    #![super::inner_attribute_func]
    pub fn foo() {}
}

#[outer_attribute_separate_module]
mod separate_module;

comptime mut global counter = 0;

comptime fn increment_counter() {
    counter += 1;
}

comptime fn outer_attribute_func(m: Module) -> Quoted {
    assert_eq(m.name(), quote { yet_another_module });
    increment_counter();
    quote { pub fn generated_outer_function() {} }
}

comptime fn inner_attribute_func(m: Module) -> Quoted {
    assert_eq(m.name(), quote { yet_another_module });
    increment_counter();
    quote { pub fn generated_inner_function() {} }
}

comptime fn outer_attribute_separate_module(m: Module) {
    assert_eq(m.name(), quote { separate_module });
    increment_counter();
}

struct Foo {}

#[add_function]
mod add_to_me {
    fn add_to_me_function() {}
}

comptime fn add_function(m: Module) {
    m.add_item(
        quote { pub fn added_function() -> super::Foo {
            add_to_me_function();
            super::Foo {}
    } }
    );
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

        // Check Module::structs
        assert_eq(foo.structs().len(), 1);
        assert_eq(bar.structs().len(), 0);

        // Check Module::name
        assert_eq(foo.name(), quote { foo });
        assert_eq(bar.name(), quote { bar });

        // Check Module::has_named_attribute
        assert(foo.has_named_attribute("some_attribute"));
        assert(foo.has_named_attribute("outer_attribute"));
        assert(!bar.has_named_attribute("some_attribute"));
        assert(another_module.has_named_attribute("some_attribute"));
    }

    assert_eq(counter, 4);

    yet_another_module::generated_outer_function();
    yet_another_module::generated_inner_function();

    let _ = add_to_me::added_function();
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
