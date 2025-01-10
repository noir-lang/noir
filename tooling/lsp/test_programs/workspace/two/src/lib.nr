use one::function_one;

pub fn function_two() {
    function_one()
}

use one::subone;

fn use_struct() {
    let _ = subone::SubOneStruct { some_field: 0, some_other_field: 2 };
}

use one::subone::SomeTrait;

fn use_global() {
    let _ = one::subone::some_global;
}

fn use_struct_method() {
    let s = subone::SubOneStruct { some_field: 0, some_other_field: 2 };
    s.foo(0, 1);
}

fn use_local_var() {
    let regular_var = 0;
    let _ = regular_var;
    let mut mutable_var = 0;
    mutable_var = 1;
}

fn use_parameter(some_param: i32) {
    let _ = some_param;
}

use one::subone::SomeAlias;

use std::default::Default;

fn use_impl_method() {
    let _: i32 = Default::default();
}

mod other;
use crate::other::other_function;
use other::another_function;

use one::subone::GenericStruct;

use std::collections::bounded_vec::BoundedVec;

fn instantiate_generic() {
    let x: BoundedVec<subone::SubOneStruct, 3> = BoundedVec::new();
}

#[attr]
pub fn foo() {}

comptime fn attr(_: FunctionDefinition) -> Quoted {
    quote { pub fn hello() {} }
}

struct Foo<T> {}

impl<U> Foo<U> {
    fn new() -> Self {
        Foo {}
    }
}

fn new_foo() -> Foo<i32> {
    Foo::new()
}

trait Bar<T, U> {
    fn bar_stuff(self);
}

impl<A> Bar<A, i32> for Foo<A> {
    fn bar_stuff(self) {}
}

fn bar_stuff() {
    let foo = Foo::new();
    foo.bar_stuff();
}

trait TraitWithDocs {
    /// Some docs
    fn foo();
}

impl TraitWithDocs for Field {
    fn foo() {}
}

