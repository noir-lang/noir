use dep::std;

trait Foo {
    fn foo() -> Field;
}


// This should not compile, as only types should have impl blocks
// TODO(https://github.com/noir-lang/noir/issues/2568): Right now we only allow structs, but arbitrary types should be allowed.
impl Foo for main {
    fn foo() -> Field {
        x + y
    }
}

fn main(x: Field, y: Field) {
}
