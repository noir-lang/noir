use dep::std::slice;
use dep::std;

// Tests nested slice passing to/from functions
unconstrained fn push_back_to_slice<T>(slice: [T], item: T) -> [T] {
    slice.push_back(item)
}

struct NestedSliceStruct {
    id: Field,
    arr: [Field]
}

unconstrained fn create_foo(id: Field, value: Field) -> NestedSliceStruct {
    let mut arr = [id];
    arr = arr.push_back(value);
    NestedSliceStruct { id, arr }
}

unconstrained fn main(a: Field, b: Field) {
    let mut slice = [create_foo(a, b), create_foo(b, a)];
    assert(slice.len() == 2);

    assert(slice[0].id == a);
    assert(slice[0].arr[0] == a);
    assert(slice[1].id == b);
    assert(slice[1].arr[1] == a);

    slice = push_back_to_slice(slice, create_foo(0, 42));
    assert(slice.len() == 3);

    assert(slice[0].id == a);
    assert(slice[0].arr[0] == a);
    assert(slice[1].id == b);
    assert(slice[1].arr[1] == a);

    assert(slice[2].id == 0);
    assert(slice[2].arr[0] == 0);
    assert(slice[2].arr[1] == 42);

    let mut remove_result = slice.remove(0);
    slice = remove_result.0;
    let mut removed_item = remove_result.1;
    assert(removed_item.arr[0] == a);

    remove_result = slice.remove(1);
    slice = remove_result.0;
    removed_item = remove_result.1;
    assert(removed_item.arr[0] == 0);

    let last_item = slice[0];

    assert(last_item.id == b);
    slice = slice.insert(1, removed_item);

    assert(slice.len() == 2);
    assert(slice[0].id == b);
    assert(slice[1].id == 0);
}
