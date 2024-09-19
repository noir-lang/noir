
trait Foo<const N: u32> {
    fn foo() {}
}

impl Foo<3> for () {
    fn foo() {}
}

fn main() {
    let _ = Foo::foo();
}
