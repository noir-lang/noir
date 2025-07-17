fn main(x: Field) -> pub Field {
    let y = comptime { 1 >> 32 };
    x + y as Field
}
