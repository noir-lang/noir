struct Wrapper<T> {
    _value: T,
}

impl<T> Wrapper<T> {
    fn new_wrapper(value: T) -> Self {
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

fn foo<T>() -> T
where
    T: MyTrait,
{
    MyTrait::new()
}

// fn verbose_but_compiles() -> MyType {
//     let a = Wrapper::new_wrapper(foo());
//     a.unwrap()
// }

// Check that are able to infer the return type of the call to `foo`
fn concise_regression() -> MyType {
    Wrapper::new_wrapper(foo()).unwrap()
    // Wrapper::unwrap(Wrapper::new_wrapper(foo()))
}

fn main() {
    // let _ = verbose_but_compiles();
    let _ = concise_regression();
}
