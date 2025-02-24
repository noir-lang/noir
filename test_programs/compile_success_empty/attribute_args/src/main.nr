#[attr_with_args(1, 2)]
#[attr_with_varargs(1, 2)]
#[attr_with_varargs(1, 2, 3, 4)]
pub struct Foo {}

comptime fn attr_with_args(s: StructDefinition, a: Field, b: Field) {
    // Ensure all variables are in scope.
    // We can't print them since that breaks the test runner.
    let _ = s;
    let _ = a;
    let _ = b;
}

#[varargs]
comptime fn attr_with_varargs(s: StructDefinition, t: [Field]) {
    let _ = s;
    for _ in t {}
    assert(t.len() < 5);
}

fn main() {}
