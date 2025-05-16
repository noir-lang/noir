use lib::Foo;

// Ensure $crate works in a crate it was not defined in
#[derive(Foo)]
struct BinStruct {}

fn main() {
    BinStruct {}.foo();
    lib::lib_fn();
}
