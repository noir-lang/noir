fn main() {
    comptime
    {
        let _expr = quote { foo(bar) }.as_expr().unwrap();
    }
}
