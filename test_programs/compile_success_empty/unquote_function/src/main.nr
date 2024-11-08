fn main() {
    bar();
}

#[output_function]
pub fn foo() {}

comptime fn output_function(_f: FunctionDefinition) -> Quoted {
    quote {
        fn bar() {}
    }
}
