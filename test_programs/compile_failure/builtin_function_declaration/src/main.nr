fn main() {
    comptime {
        let _a = id(3);
    }
}

fn id(x: Field) -> Field {
    x
}