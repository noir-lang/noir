mod foo {
    mod bar {}
}

fn main() {
    comptime {
        let m = quote { foo::bar }.as_module();
        let _ = m.unwrap();
    }
}
