use dep::std::slice;
use dep::std;
fn main(x : Field, y : pub Field) {
    let slice = [0; 2];
    let slice = if x != y {
        if x != 20 {
            slice.push_back(y)
        } else {
            slice
        }
    } else {
        slice
    };
    assert(slice[2] == 10);
    assert(slice.len() == 3);

    let mut slice = [0; 2];
    if x != y {
        slice = slice.push_back(y);
        slice = slice.push_back(x);
    } else {
        slice = slice.push_back(x);
    }
    assert(slice[3] == 5);
    assert(slice.len() == 4);

    // TODO: Panic with "Expected array value" as the slice_push_back
    // is still an instruction that has not been correctly loaded
    // let mut slice = [0; 2];
    // if x != y {
    //     if x != 20 {
    //         slice = slice.push_back(y);
    //     } else {
    //         slice = slice.push_back(x);
    //     }
    // }
    // assert(slice[2] == 10);
    
    let mut slice = [0; 2];
    slice = slice.push_back(x);

    let mut new_slice = [];
    for i in 0..5 {
        new_slice = new_slice.push_back(i);
    }
    assert(new_slice.len() == 5);

    new_slice = new_slice.push_front(20);
    assert(new_slice[0] == 20);
    assert(new_slice.len() == 6);

    let (popped_slice, last_elem) = new_slice.pop_back();
    assert(last_elem == 4);
    assert(popped_slice.len() == 5);

    let (first_elem, rest_of_slice) = popped_slice.pop_front();
    assert(first_elem == 20);
    assert(rest_of_slice.len() == 4);

    new_slice = rest_of_slice.insert(2, 100);
    assert(new_slice[2] == 100);
    assert(new_slice[4] == 3);
    assert(new_slice.len() == 5);

    let (remove_slice, removed_elem) = new_slice.remove(3);
    assert(removed_elem == 2);
    assert(remove_slice[3] == 3);
    assert(remove_slice.len() == 4);

    regression_2083();
}

// Ensure that slices of struct/tuple values work.
fn regression_2083() {
    let y = [(1, 2)];
    let y = y.push_back((3, 4)); // [(1, 2), (3, 4)]
    let y = y.push_back((5, 6)); // [(1, 2), (3, 4), (5, 6)]
    assert(y[2].1 == 6);

    let y = y.push_front((10, 11)); // [(10, 11), (1, 2), (3, 4), (5, 6)]
    let y = y.push_front((12, 13)); // [(12, 13), (10, 11), (1, 2), (3, 4), (5, 6)]

    assert(y[1].0 == 10);

    let y = y.insert(1, (55, 56)); // [(12, 13), (55, 56), (10, 11), (1, 2), (3, 4), (5, 6)]
    assert(y[0].1 == 13);
    assert(y[1].1 == 56);
    assert(y[2].0 == 10);

    let (y, x) = y.remove(2); // [(12, 13), (55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y[2].0 == 1);
    assert(x.0 == 10);
    assert(x.1 == 11);

    let (x, y) = y.pop_front(); // [(55, 56), (1, 2), (3, 4), (5, 6)]
    assert(y[0].0 == 55);
    assert(x.0 == 12);
    assert(x.1 == 13);

    let (y, x) = y.pop_back(); // [(55, 56), (1, 2), (3, 4)]
    assert(y.len() == 3);
    assert(x.0 == 5);
    assert(x.1 == 6);
}

// // TODO: this is still broken using the new mem2reg pass
// fn main(x : Field, y : pub Field) {
//     let q = &mut &mut 0;
//     let q1 = *q;
//     let q2 = *q;

//     if x != y {
//         *q1 = 1;
//         // mem2reg will know q1 = 1, but not that q2 = 1
//         *q2 = 2; // now we'd expect q1 == q2 == 2, but mem2reg now thinks x1 = 1 & x2 = 2
        
//         // This now passes and we've broken mem2reg :(
//         assert(*q1 == 1);
//     }
// }
