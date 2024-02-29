fn main(input: [u32; 4]) {
    let mut slice1: [u32] = [1, 2, 3, 4];
    if slice1[0] == 3 {
        slice1[1] = 4;
    }

    if slice1[1] == 5 {
        slice1[3] = 6;
    }

    for i in 0..4 {
        assert(slice1[i] == input[i]);
    }
}
