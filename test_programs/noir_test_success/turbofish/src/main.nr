
struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

impl<T> Bar<T> {
    // This is to test that we can use turbofish on methods as well
    fn zeroed<A>(_self: Self) -> A {
        dep::std::unsafe::zeroed()
    }
}

fn foo<T>(bar: Bar<T>) {
    assert(bar.one == bar.two);
}

global bar1: Bar<Field> = Bar { one: 2, two: 2, other: 0 };
global bar2: Bar<u32> = Bar { one: 2, two: 2, other: 10 };

#[test]
fn test_plain_function_call() {
    // let bar1: Bar<Field> = Bar { one: 1, two: 2, other: 0 };

    // Test turbofish operator
    foo::<Field>(bar1);
}

#[test]
fn test_method_call_with_implicit_generics() {
    // assert(bar1.zeroed::<u32>() == bar2.other);
    assert(bar1.zeroed::<u32>() == 0);
}