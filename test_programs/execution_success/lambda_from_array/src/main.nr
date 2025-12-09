// Many parts of the code here are regressions from issue #5503 (https://github.com/noir-lang/noir/issues/5503)
fn main(x: u32) {
    lambdas_in_array_literal(x - 1);
    lambdas_in_array_literal(x);
    lambdas_in_array_literal(x + 2);
    lambdas_in_array_literal(x + 1);

    lambdas_in_slice_literal(x - 1);
    lambdas_in_slice_literal(x);
    lambdas_in_slice_literal(x + 1);
    lambdas_in_slice_literal(x + 2);

    functions_in_array_literal(x - 1);
    functions_in_array_literal(x);
    functions_in_slice_literal(x - 1);
    functions_in_slice_literal(x);

    let example_lambda: fn(u8) -> u8 = |x| x + 1;
    let lambdas: [fn(u8) -> u8; 8] = [example_lambda; 8];
    println(lambdas[0](5));
    // Dynamic dispatch
    println(lambdas[x - 1](5));

    let lambdas: [fn(()) -> (); 1] = [|_: ()| {}];
    lambdas[0](());
    lambdas[x - 1](());

    // Also check against slices
    let lambdas: [fn(()) -> ()] = &[|_: ()| {}];
    lambdas[0](());
    lambdas[x - 1](());
}

fn lambdas_in_array_literal(x: u32) {
    let xs = [|| println("hi"), || println("bye"), || println("wow"), || println("big")];
    (xs[x])();
}

fn lambdas_in_slice_literal(x: u32) {
    let xs = &[|| println("hi"), || println("bye"), || println("big"), || println("wow")];
    (xs[x])();
}

fn functions_in_array_literal(x: u32) {
    let xs = [foo, bar];
    (xs[x])();
}

fn functions_in_slice_literal(x: u32) {
    let xs = &[baz, qux];
    (xs[x])();
}

fn foo() {
    println("hi");
}
fn bar() {
    println("bye");
}
fn baz() {
    println("hi");
}
fn qux() {
    println("bye");
}
