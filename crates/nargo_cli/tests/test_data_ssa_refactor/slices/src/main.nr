use dep::std::slice;
use dep::std;
fn main(x : Field, y : pub Field) {
    /// TODO(#1889): Using slices in if statements where the condition is a witness
    /// is not yet supported

    let mut slice: [Field] = [0; 2];
    assert(slice[0] == 0);
    assert(slice[0] != 1);
    slice[0] = x;
    assert(slice[0] == x);

    let slice_plus_10 = slice.push_back(y);
    assert(slice_plus_10[2] == 10);
    assert(slice_plus_10[2] != 8);
    assert(slice_plus_10.len() == 3);

    let mut new_slice: [Field] = [];
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
}

