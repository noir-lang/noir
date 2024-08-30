global N = 10;

unconstrained fn main() {
    let mut arr = [0; N];
    let mut mid_change = arr;

    for i in 0..N {
        if i == N / 2 {
            // println("inside if");
            // println(arr);
            mid_change = arr;
            // println(mid_change);
        }
        // println(i);
        // println(mid_change);
        arr[i] = 27;
    }

    // Expect:
    // arr        = [27, 27, 27, 27, 27, 27, 27, 27, 27, 27]
    // mid_change = [27, 27, 27, 27, 27, 0, 0, 0, 0, 0]

    let modified_i = N / 2 + 1;
    assert_eq(arr[modified_i], 27);

    // Fail here!
    assert(mid_change[modified_i] != 27);
}
