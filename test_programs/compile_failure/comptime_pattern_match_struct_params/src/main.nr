// Regression for #10914 (https://github.com/noir-lang/noir/issues/10914)
struct Foo {
    msg: str<5>,
    value: u32,
}
struct Bar {
    msg: str<5>,
    value: u32,
}
fn main() {
    comptime {
        // We expect a single type error and for nothing to be printed as the type
        // supplied is incorrect
        print_foo(Bar { msg: "Hello", value: 123 })
    }
}

fn print_foo(Foo { msg, value }: Foo) {
    println(f"{msg} {value}");
}
