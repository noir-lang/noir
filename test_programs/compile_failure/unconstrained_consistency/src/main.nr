#[mark_unconstrained]
fn sneaky() {
    helper();
}

unconstrained fn helper() {}

comptime fn mark_unconstrained(f: FunctionDefinition) {
    f.set_unconstrained(true);
}

fn main() {
    let funcs = [sneaky];
    funcs[0]();
}
