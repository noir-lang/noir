#[foo]
comptime fn foo(_: FunctionDefinition) {
    let m = dependency::modules[0];
    m.add_item(quote { pub fn bar() {} });
}

fn main() {
    dependency::foo::bar()
}
