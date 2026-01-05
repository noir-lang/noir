struct foo {
    a: u32,
    b: [Field; 2],
}

fn main(cond: bool) -> pub u32 {
    let mut vector = #[foo { a: 2, b: [3, 4] }];
    if cond {
        vector = vector.push_back(foo { a: 5, b: [6, 7] });
    }
    vector = vector.insert(vector.len(), foo { a: 8, b: [9, 10] });
    vector = vector.insert(vector.len(), foo { a: 8, b: [9, 10] });
    vector.len()
}
