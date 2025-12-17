fn main(array: [Field; 1], cond: bool) -> pub u32 {
    let list = array.as_list();
    let mut list = std::hint::black_box(list);
    if cond {
        list = list.push_back(1);
    }
    list.len()
}
