// A type-level `Field` division by zero must be rejected rather than silently
// canonicalized to zero (the field-element inverse of zero is zero).
struct W<let N: Field> {}

fn foo<let N: Field>(_x: W<N>) -> W<N / (N - N)> {
    W {}
}

fn main() {
    let w_5: W<5Field> = W {};
    // error: Division by zero: 0x05 / 0x00
    let _w = foo(w_5);
}
