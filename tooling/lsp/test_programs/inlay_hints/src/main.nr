fn main() {
    let var = 0;
}

struct Foo {

}

fn make_foo() -> Foo {
    Foo {}
}

fn foo() {
    let foo = make_foo();
}

fn test_for() {
    for i in 0..10 {}
}

global var = 0;

fn test_fn(one: i32, two: i32) {}

fn call_test_fn() {
    test_fn(1, 2);
}

struct SomeStruct {

}

impl SomeStruct {
    fn some_method(self, one: i32) {}
}

fn call_method() {
    let s = SomeStruct {};
    s.some_method(1);
}

