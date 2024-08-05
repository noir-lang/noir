mod foo {}

fn main() {
    comptime
    {
        let foo = quote { foo }.as_module().unwrap();
    }
}
