fn main(a: Field, b: bool) -> pub u64 {
    let c = b as Field;
    let res = a - c;
    res as u64
}
