fn main(a: u32) -> pub Field {
    foo()[a].1 as Field
}

fn foo() -> [([u64; 0], u16); 1] {
    [([], 10)]
}
