// Every field of the element occupies no ACIR cells, so the array has no memory for a
// memory operation's own check to report against, and the explicit check is the whole of
// the access. It still names the index and length the program used.
struct Z {
    a: [Field; 0],
    b: str<0>,
}

fn main(i: u32, x: Field) -> pub Field {
    let arr = [Z { a: [], b: "" }, Z { a: [], b: "" }, Z { a: [], b: "" }, Z { a: [], b: "" }];
    let _ = arr[i];
    x
}
