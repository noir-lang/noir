#[foo]
fn main() {}

comptime fn foo(_f: FunctionDefinition) -> Quoted {
    let t = quote { Field }.as_type();
    // We parse 0 or more top level expressions from attribute output
    // so for invalid input we previously "successfully" parsed 0 items.
    quote { use $t; }
}
