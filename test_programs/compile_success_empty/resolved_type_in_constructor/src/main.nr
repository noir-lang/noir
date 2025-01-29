fn main() {
    let _ = my_macro!();
}

comptime fn my_macro() -> Quoted {
    let typ = quote { Foo }.as_type();
    quote[$typ {}]
}

struct Foo {}
