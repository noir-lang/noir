// A comptime-mutable global observed at runtime must expand to its final mutated value,
// not its initializer: the attribute that mutates it is already expanded away, so
// re-evaluating the initializer would produce a different program.
comptime mut global COUNTER: u32 = 0;

comptime fn bump(_f: FunctionDefinition) {
    COUNTER += 1;
}

#[bump]
fn foo() {}

fn main(x: Field) {
    foo();
    let counter = comptime { COUNTER };
    assert(counter == 1);
    assert(x == 5);
}
