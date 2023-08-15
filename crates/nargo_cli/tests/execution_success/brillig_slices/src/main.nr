use dep::std::slice;
use dep::std;

unconstrained fn main(x: Field, y: Field) {
    let mut slice: [Field] = [y, x];
    assert(slice.len() == 2);

    slice = slice.push_back(7);
    assert(slice.len() == 3);
    assert(slice[0] == y);
    assert(slice[1] == x);
    // dep::std::println(slice[2]);
    assert(slice[2] == 7);

    // // Array set on slice target
    // slice[0] = x;
    // slice[1] = y;
    // slice[2] = 1;

    // assert(slice[0] == x);
    // assert(slice[1] == y);
    // assert(slice[2] == 1);

    // slice = push_front_to_slice(slice, 2);
    // assert(slice.len() == 4);
    // assert(slice[0] == 2);
    // assert(slice[1] == x);
    // assert(slice[2] == y);
    // assert(slice[3] == 1);

    // let (item, popped_front_slice) = slice.pop_front();
    // slice = popped_front_slice;
    // assert(item == 2);

    // assert(slice.len() == 3);
    // assert(slice[0] == x);
    // assert(slice[1] == y);
    // assert(slice[2] == 1);

    // let (popped_back_slice, another_item) = slice.pop_back();
    // slice = popped_back_slice;
    // assert(another_item == 1);

    // assert(slice.len() == 2);
    // assert(slice[0] == x);
    // assert(slice[1] == y);

    // slice = slice.insert(1, 2);
    // assert(slice.len() == 3);
    // assert(slice[0] == x);
    // assert(slice[1] == 2);
    // assert(slice[2] == y);

    // let (removed_slice, should_be_2) = slice.remove(1);
    // slice = removed_slice;
    // assert(should_be_2 == 2);

    // assert(slice.len() == 2);
    // assert(slice[0] == x);
    // assert(slice[1] == y);

    // let (slice_with_only_x, should_be_y) = slice.remove(1);
    // slice = slice_with_only_x;
    // assert(should_be_y == y);

    // assert(slice.len() == 1);
    // assert(slice[0] == x);

    // let (empty_slice, should_be_x) = slice.remove(0);
    // assert(should_be_x == x);
    // assert(empty_slice.len() == 0);
}

// Tests slice passing to/from functions
unconstrained fn push_front_to_slice<T>(slice: [T], item: T) -> [T] {
    slice.push_front(item)
}
