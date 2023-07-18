use dep::std::slice;
use dep::std;

unconstrained fn main(x: Field, y: Field) {
    //Get the slice from a function to correct the type until https://github.com/noir-lang/noir/issues/1931 is fixed
    let mut slice: [Field] = get_me_a_slice(y, x);
    assert(slice.len() == 2);

    slice = slice.push_back(7);
    assert(slice.len() == 3);
    assert(slice[0] == y);
    assert(slice[1] == x);
    assert(slice[2] == 7);

    // Array set on slice target
    slice[0] = x;
    slice[1] = y;
    slice[2] = 1;

    assert(slice[0] == x);
    assert(slice[1] == y);
    assert(slice[2] == 1);

    slice = slice.push_front(2);
    assert(slice.len() == 4);
    assert(slice[0] == 2);
    assert(slice[1] == x);
    assert(slice[2] == y);
    assert(slice[3] == 1);

    let (item, popped_front_slice) = slice.pop_front();
    assert(item == 2);
    slice = popped_front_slice;

    assert(slice.len() == 3);
    assert(slice[0] == x);
    assert(slice[1] == y);
    assert(slice[2] == 1);

    let (popped_back_slice, another_item) = slice.pop_back();
    assert(another_item == 1);
    slice = popped_back_slice;

    assert(slice.len() == 2);
    assert(slice[0] == x);
    assert(slice[1] == y);

    slice = slice.insert(1, 2);
    assert(slice.len() == 3);
    assert(slice[0] == x);
    assert(slice[1] == 2);
    assert(slice[2] == y);

    let (removed_slice, should_be_2) = slice.remove(1);
    assert(should_be_2 == 2);
    slice = removed_slice;

    assert(slice.len() == 2);
    assert(slice[0] == x);
    assert(slice[1] == y);

    let (slice_with_only_x, should_be_y) = slice.remove(1);
    assert(should_be_y == y);
    slice = slice_with_only_x;

    assert(slice.len() == 1);
    assert(removed_slice[0] == x);

    let (empty_slice, should_be_x) = slice.remove(0);
    assert(should_be_x == x);
    assert(empty_slice.len() == 0);

    
    let sequence = create_sequence([], 5);

    assert(sequence.len() == 5);
    assert(sequence[0] == 5);
    assert(sequence[1] == 4);
    assert(sequence[2] == 3);
    assert(sequence[3] == 2);
    assert(sequence[4] == 1);
}

unconstrained fn get_me_a_slice(x: Field, y: Field) -> [Field] {
    [x, y]
}

unconstrained fn create_sequence(mut sequence: [Field], n: Field) -> [Field] {
    if n != 0 {
        sequence = create_sequence(sequence.push_back(n), n - 1);
    }

    sequence
}