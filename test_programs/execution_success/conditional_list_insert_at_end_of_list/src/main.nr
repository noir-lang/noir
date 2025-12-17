struct foo {
    a: u32,
    b: [Field; 2],
}

fn main(cond: bool) -> pub u32 {
    let mut list = &[foo { a: 2, b: [3, 4] }];
    if cond {
        list = list.push_back(foo { a: 5, b: [6, 7] });
    }
    list = list.insert(list.len(), foo { a: 8, b: [9, 10] });
    list.len()
}
