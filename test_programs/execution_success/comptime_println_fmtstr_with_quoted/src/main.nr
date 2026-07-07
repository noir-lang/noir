#[world]
comptime fn world(f: FunctionDefinition) {
    let name = f.name();
    println(f"hello {name}");
}

fn main() {}
