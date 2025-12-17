fn main(input: [u32; 4]) {
    let mut list1: [u32] = &[1, 2, 3, 4];
    if list1[0] == 3 {
        list1[1] = 4;
    }

    if list1[1] == 5 {
        list1[3] = 6;
    }

    for i in 0..4 {
        assert(list1[i] == input[i]);
    }
}
