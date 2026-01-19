// Regression for #10918 (https://github.com/noir-lang/noir/issues/10918)
struct Foo {
    a: u32,
    b: bool,
}
fn main() {
    comptime {
        let x = Foo { a: 1, a: "hello", c: true };
        // We expect nothing to be printed as we have invalid input
        println(x);
        println(x.c);
    }
}