// The element has two fields but only one of them occupies an ACIR cell.
struct S {
    a: [Field; 0],
    b: Field,
}

fn main(i: u32, x: Field) -> pub Field {
    let mut arr = [S { a: [], b: x }, S { a: [], b: x + 1 }, S { a: [], b: x + 2 }, S { a: [], b: x + 3 }];
    arr[i].a = [];
    arr[0].b
}
