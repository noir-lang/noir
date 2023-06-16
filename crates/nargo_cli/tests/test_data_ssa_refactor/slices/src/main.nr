use dep::std::slice;

fn main(x : Field, y : pub Field) {

    let mut slice: [Field] = [0; 2];

    assert(slice[0] == 0);
    assert(slice[0] != 1);

    let slice_plus_10 = slice.push_back(10);
    assert(slice_plus_10[2] == 10);
    assert(slice_plus_10[2] != 8);
    assert(slice_plus_10.len() == 3);

    // TODO
    // let mut new_slice: [Field] = [];
    // for i in 0..5 {
    //     new_slice = new_slice.push_back(0);
    // }
    // assert(new_slice.len() == 5);
}

