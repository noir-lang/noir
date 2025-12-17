fn main() {
    let empty_list: [u8] = &[];

    if empty_list != &[] {
        let _ = empty_list.pop_front();
    }

    if empty_list.len() != 0 {
        let _ = empty_list.pop_front();
    }
}
