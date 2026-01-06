#['outer_attribute]
mod foo {
    #!['some_attribute]
    pub fn x() {}
    pub fn y() {}
    pub fn z() {}

    pub struct Struct1 {}
    pub struct Struct2 {}
    pub struct Struct3 {}
}

contract bar {}

#['some_attribute]
mod another_module {}

#[outer_attribute_func]
mod yet_another_module {
    #![super::inner_attribute_func]
    pub fn foo() {}
}

#[outer_attribute_separate_module]
mod separate_module;

comptime mut global counter: u32 = 0;

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

pub struct Foo {}

#[add_function]
mod add_to_me {
    fn add_to_me_function() {}
}

comptime fn add_function(m: Module) {
    m.add_item(
        quote { pub fn added_function() -> super::Foo {
            add_to_me_function();
            super::Foo {}
    } },
    );
}

fn main() {
    let _ = foo::Struct1 {};

    comptime {
        // Check Module::is_contract
        let foo = quote { foo }.as_module().unwrap();
        assert(!foo.is_contract());

        let bar = quote { bar }.as_module().unwrap();
        assert(bar.is_contract());

        let another_module = quote { another_module }.as_module().unwrap();

        // Check Module::functions
        let foo_functions = foo.functions();
        assert_eq(foo_functions.len(), 3);
        assert_eq(foo_functions[0].name(), quote { x });
        assert_eq(foo_functions[1].name(), quote { y });
        assert_eq(foo_functions[2].name(), quote { z });

        assert_eq(bar.functions().len(), 0);

        // Check Module::structs
        let foo_structs = foo.structs();
        assert_eq(foo_structs.len(), 3);
        assert_eq(foo_structs[0].name(), quote { Struct1 });
        assert_eq(foo_structs[1].name(), quote { Struct2 });
        assert_eq(foo_structs[2].name(), quote { Struct3 });
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
    pub mod qux {}
}

#[test]
fn as_module_test() {
    comptime {
        let my_mod = quote { baz::qux }.as_module().unwrap();
        assert_eq(my_mod.name(), quote { qux });
    }
}
// docs:end:as_module_example

// docs:start:parent_example
mod module1 {
    pub mod module2 {}
}

#[test]
fn parent_test() {
    comptime {
        let my_module2 = quote [module1::module2].as_module().unwrap();
        assert_eq(my_module2.name(), quote [module2]);

        let my_module1 = my_module2.parent().unwrap();
        assert_eq(my_module1.name(), quote [module1]);

        // The top-level module in each crate has no name
        let top_level_module = my_module1.parent().unwrap();
        assert_eq(top_level_module.name(), quote []);
    }
}
// docs:end:parent_example

// docs:start:child_modules_example
mod my_module {
    pub mod child1 {}
    pub mod child2 {}
    pub mod child3 {
        pub mod nested_child {}
    }
}

#[test]
fn child_modules_test() {
    comptime {
        let my_module = quote [my_module].as_module().unwrap();
        let children = my_module.child_modules().map(Module::name);

        // The order children are returned in is left unspecified.
        assert_eq(children, [quote [child1], quote [child2], quote [child3]].as_vector());
    }
}
// docs:end:child_modules_example

/// A longer test using all submodules in this crate. This shows that modules
/// declaring separate files are currently listed before modules defined in the
/// same file. This is not desired so the order for `child_modules` is left unspecified
/// so that this can be fixed in the future.
#[test]
fn child_modules_test2() {
    comptime {
        let top = quote [my_module].as_module().unwrap().parent().unwrap();
        let children = top.child_modules().map(Module::name);

        let expected = [
            quote [separate_module],
            quote [foo],
            quote [bar],
            quote [another_module],
            quote [yet_another_module],
            quote [add_to_me],
            quote [baz],
            quote [module1],
            quote [my_module],
        ].as_vector();
        assert_eq(children, expected);
    }
}

/// Ensures calling `Module::name` on the top-level module does not panic
#[test]
fn top_level_module_name_is_empty() {
    comptime {
        let top = quote [my_module].as_module().unwrap().parent().unwrap();
        assert_eq(top.name(), quote []);
    }
}
