use dep::std::slice;
use dep::std;

unconstrained fn main(x: Field, y: Field) {
    // Mark it as mut so the compiler doesn't simplify the following operations
    // But don't reuse the mut slice variable until this is fixed https://github.com/noir-lang/noir/issues/1931
    let slice: [Field] = [y, x];
    assert(slice.len() == 2);

    let mut pushed_back_slice = slice.push_back(7);
    assert(pushed_back_slice.len() == 3);
    assert(pushed_back_slice[0] == y);
    assert(pushed_back_slice[1] == x);
    assert(pushed_back_slice[2] == 7);

    // Array set on slice target
    pushed_back_slice[0] = x;
    pushed_back_slice[1] = y;
    pushed_back_slice[2] = 1;

    assert(pushed_back_slice[0] == x);
    assert(pushed_back_slice[1] == y);
    assert(pushed_back_slice[2] == 1);

    assert(slice.len() == 2);

    let pushed_front_slice = pushed_back_slice.push_front(2);
    assert(pushed_front_slice.len() == 4);
    assert(pushed_front_slice[0] == 2);
    assert(pushed_front_slice[1] == x);
    assert(pushed_front_slice[2] == y);
    assert(pushed_front_slice[3] == 1);

    let (item, popped_front_slice) = pushed_front_slice.pop_front();
    assert(item == 2);

    assert(popped_front_slice.len() == 3);
    assert(popped_front_slice[0] == x);
    assert(popped_front_slice[1] == y);
    assert(popped_front_slice[2] == 1);

    let (popped_back_slice, another_item) = popped_front_slice.pop_back();
    assert(another_item == 1);

    assert(popped_back_slice.len() == 2);
    assert(popped_back_slice[0] == x);
    assert(popped_back_slice[1] == y);

    let inserted_slice = popped_back_slice.insert(1, 2);
    assert(inserted_slice.len() == 3);
    assert(inserted_slice[0] == x);
    assert(inserted_slice[1] == 2);
    assert(inserted_slice[2] == y);

    let (removed_slice, should_be_2) = inserted_slice.remove(1);
    assert(should_be_2 == 2);

    assert(removed_slice.len() == 2);
    assert(removed_slice[0] == x);
    assert(removed_slice[1] == y);

    let (slice_with_only_x, should_be_y) = removed_slice.remove(1);
    assert(should_be_y == y);

    assert(slice_with_only_x.len() == 1);
    assert(removed_slice[0] == x);

    let (empty_slice, should_be_x) = slice_with_only_x.remove(0);
    assert(should_be_x == x);
    assert(empty_slice.len() == 0);
}

