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
    one: i32,
}

impl SomeStruct {
    fn some_method(self, one: i32) {}
}

fn call_method() {
    let s = SomeStruct { one: 1 };
    s.some_method(1);
}

fn call_where_name_matches() {
    let one = 1;
    let two = 2;
    test_fn(one, two);
}

fn call_where_member_name_matches() {
    let s = SomeStruct { one: 1 };
    let two = 2;
    test_fn(s.one, two);
}

fn one() -> i32 {
    1
}

fn call_where_call_matches_name() {
    let two = 2;
    test_fn(one(), two);
}

fn with_arg(arg: i32) {}

fn call_with_arg() {
    let x = 1;
    with_arg(x);
}

fn with_underscore(_x: i32) {}

fn call_with_underscore() {
    with_underscore(1);
}

fn one_arg_with_one_char(x: i32) {}

fn call_one_arg_with_one_char() {
    one_arg_with_one_char(1);
}

fn one_arg_with_obvious_name(other: i32) {}

fn call_one_arg_with_obvious_name() {
    one_arg_with_obvious_name(1);
}

fn yet_another_function(name: i32) {}

fn call_yet_another_function() {
    let some_name = 1;
    yet_another_function(some_name)
}

