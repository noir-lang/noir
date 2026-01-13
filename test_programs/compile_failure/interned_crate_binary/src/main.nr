use lib::Foo;

// Accessing the private method via $crate should not work from a different crate.
#[derive(Foo)]
struct BinStruct {}

fn main() {
    BinStruct {}.foo();
    lib::lib_fn();
}
