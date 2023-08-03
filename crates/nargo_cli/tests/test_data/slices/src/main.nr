use dep::std::slice;
use dep::std;
fn main(x : Field, y : pub Field) {
    /// TODO(#1889): Using slices in if statements where the condition is a witness
    /// is not yet supported

    // let mut slice = [0; 2];
    // if x != y {
    //     slice = slice.push_back(y);
    // } else {
    //     slice = slice.push_back(x);
    // }
    // assert(slice[2] == 10);

    // let mut slice = [0; 2];
    // slice = slice.push_back(x);

    let mut arr = [0; 2];
    if x != y {
        arr = [5, 10];
    } else {
        arr = [10, 5];
    }
    assert(arr[0] == 5);

    // let mut new_slice = [];
    // for i in 0..5 {
    //     new_slice = new_slice.push_back(i);
    // }
    // assert(new_slice.len() == 5);

    // new_slice = new_slice.push_front(20);
    // assert(new_slice[0] == 20);
    // assert(new_slice.len() == 6);

    // let (popped_slice, last_elem) = new_slice.pop_back();
    // assert(last_elem == 4);
    // assert(popped_slice.len() == 5);

    // let (first_elem, rest_of_slice) = popped_slice.pop_front();
    // assert(first_elem == 20);
    // assert(rest_of_slice.len() == 4);

    // new_slice = rest_of_slice.insert(2, 100);
    // assert(new_slice[2] == 100);
    // assert(new_slice[4] == 3);
    // assert(new_slice.len() == 5);

    // let (remove_slice, removed_elem) = new_slice.remove(3);
    // assert(removed_elem == 2);
    // assert(remove_slice[3] == 3);
    // assert(remove_slice.len() == 4);
}

