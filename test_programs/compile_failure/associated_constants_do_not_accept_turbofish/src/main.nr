pub trait Foo {
    let N: u32;

    fn foo() -> [Field; Self::N::<i32>] {
        [0; Self::N::<i32>]
    }
}

fn main() {}
