mod foo {}

contract bar {}

fn main() {
    comptime
    {
        let foo = quote { foo }.as_module().unwrap();
        assert(!foo.is_contract());

        let bar = quote { bar }.as_module().unwrap();
        assert(bar.is_contract());
    }
}
