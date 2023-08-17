use dep::std::slice;
use dep::std;
fn main(x : Field, y : pub Field) {
    /// TODO(#1889): Using slices in if statements where the condition is a witness
    /// is not yet supported

    let mut slice = [0; 2];
    assert(slice[0] == 0);
    assert(slice[0] != 1);
    slice[0] = x;
    assert(slice[0] == x);

    let slice_plus_10 = slice.push_back(y);
    assert(slice_plus_10[2] == 10);
    assert(slice_plus_10[2] != 8);
    assert(slice_plus_10.len() == 3);

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

    let append = [1, 2].append([3, 4, 5]);
    assert(append.len() == 5);
    assert(append[0] == 1);
    assert(append[4] == 5);

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
