#[attr_with_args(a b, c d)]
#[varargs(one, two)]
#[varargs(one, two, three, four)]
struct Foo {}

comptime fn attr_with_args(s: StructDefinition, a: Quoted, b: Quoted) {
    // Ensure all variables are in scope.
    // We can't print them since that breaks the test runner.
    let _ = s;
    let _ = a;
    let _ = b;
}

comptime fn varargs(s: StructDefinition, t: [Quoted]) {
    let _ = s;
    for _ in t {}
    assert(t.len() < 5);
}

fn main() {}
