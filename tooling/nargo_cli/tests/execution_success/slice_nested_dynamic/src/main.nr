use dep::std::println;

fn main(y: pub Field) {
    let small_inner: [Field] = [1, 1, 1];
    let big_inner: [Field] = [2, 2, 2, 2];
    let nested_slice: [[Field]] = [small_inner, big_inner];
    assert(nested_slice[y - 3].len() == 3);
    assert(nested_slice[y - 2].len() == 4);

    let mut double_nested_slice: [[[Field]]] = [nested_slice, nested_slice];
    println(double_nested_slice[y - 3].len());
    assert(double_nested_slice[y - 3].len() == 2);
    assert(double_nested_slice[y - 2].len() == 2);

    double_nested_slice[y - 3] = double_nested_slice[y - 3].push_back(small_inner);
    println(double_nested_slice[y - 3].len());
    assert(double_nested_slice[y - 3].len() == 3);
    assert(double_nested_slice[y - 3][y - 1].len() == 3);
    assert(double_nested_slice[y - 3][y - 1][0] == 1);
    assert(double_nested_slice[y - 3][y - 1][1] == 1);
    assert(double_nested_slice[y - 3][y - 1][2] == 1);


    // double_nested_slice[1] = double_nested_slice[1].push_back(small_inner);
    // println(double_nested_slice[y - 2].len());
    // assert(double_nested_slice[y - 2].len() == 4);

    // // We should have a nested max slice sizes structure of [2, 5, 4]
    // // after this push back
    // // TODO: It is giving us 5 for the further nested slices
    // double_nested_slice[1] = double_nested_slice[1].push_back(small_inner);
    // println(double_nested_slice[y - 2].len());
    // assert(double_nested_slice[y - 2].len() == 5);

}