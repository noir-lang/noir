fn main(x: u32) {
    lambdas_no_inputs(x);
    println(lambdas_with_input_and_return_values(x - 1));
    println(lambdas_with_input_and_return_values(x));
    println(lambdas_with_input_and_return_values(x + 1));
}
fn lambdas_no_inputs(x: u32) {
    let f1 = || println("hi");
    let f2 = || println("bye");
    let f = if x == 0 {
        f1
    } else if x == 1 {
        f2
    } else {
        panic(f"!")
    };
    f();
}
fn lambdas_with_input_and_return_values(x: u32) -> u32 {
    let f1 = |value: u32| value;
    let f2 = |value: u32| value + 1;
    let f3 = |value: u32| value + 2;
    let f = if x == 0 {
        f1
    } else if x == 1 {
        f2
    } else {
        f3
    };
    f(x)
}
