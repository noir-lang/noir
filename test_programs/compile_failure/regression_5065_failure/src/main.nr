struct Wrapper<T> {
    _value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Self { _value: value }
    }

    fn unwrap(self) -> T {
        self._value
    }
}

trait MyTrait {
    fn new() -> Self;
}

struct MyType {}

impl MyTrait for MyType {
    fn new() -> Self {
        MyType {}
    }
}

fn foo<T>() -> T where T: MyTrait {
    MyTrait::new()
}

struct BadType {}

// Check that we get "No matching impl found for `BadType: MyTrait`"
fn concise_regression() -> BadType {
    Wrapper::new(foo()).unwrap()
}

fn main() {
    let _ = concise_regression();
}
