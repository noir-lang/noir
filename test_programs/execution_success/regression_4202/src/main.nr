fn main(input: [u32; 4]) {
    let mut vector1: [u32] = [1, 2, 3, 4].as_vector();
    if vector1[0] == 3 {
        vector1[1] = 4;
    }

    if vector1[1] == 5 {
        vector1[3] = 6;
    }

    for i in 0..4 {
        assert(vector1[i] == input[i]);
    }
}
