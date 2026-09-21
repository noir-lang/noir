// Writing the field that occupies no cell still has to check the index it was written at.
struct S {
    a: [Field; 0],
    b: Field,
}

fn main(i: u32, x: Field) -> pub Field {
    let mut arr = [S { a: [], b: x }, S { a: [], b: x + 1 }, S { a: [], b: x + 2 }, S { a: [], b: x + 3 }];
    arr[i].a = [];
    arr[0].b
}
