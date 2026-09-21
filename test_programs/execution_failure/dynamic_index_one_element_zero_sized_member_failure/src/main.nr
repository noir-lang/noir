// A one-element array resolves its access to the only index in bounds, so the memory
// operation disappears and the explicit check is the only report the access gets.
struct S {
    a: [Field; 0],
    b: Field,
}

fn main(i: u32, x: Field) -> pub Field {
    let arr = [S { a: [], b: x }];
    arr[i].b
}
