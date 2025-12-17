use std::list;
// Tests nested list passing to/from functions
unconstrained fn push_back_to_list<T>(list: [T], item: T) -> [T] {
    list.push_back(item)
}

struct NestedListStruct {
    id: Field,
    arr: [Field]
}

unconstrained fn create_foo(id: Field, value: Field) -> NestedListStruct {
    let mut arr = [id];
    arr = arr.push_back(value);
    NestedListStruct { id, arr }
}

unconstrained fn main(a: Field, b: Field) {
    let mut list = [create_foo(a, b), create_foo(b, a)];
    assert(list.len() == 2);

    assert(list[0].id == a);
    assert(list[0].arr[0] == a);
    assert(list[1].id == b);
    assert(list[1].arr[1] == a);

    list = push_back_to_list(list, create_foo(0, 42));
    assert(list.len() == 3);

    assert(list[0].id == a);
    assert(list[0].arr[0] == a);
    assert(list[1].id == b);
    assert(list[1].arr[1] == a);

    assert(list[2].id == 0);
    assert(list[2].arr[0] == 0);
    assert(list[2].arr[1] == 42);

    list = list.push_front(create_foo(1, 43));
    list = list.push_back(create_foo(2, 44));

    assert(list.len() == 5);

    let pop_front_result = list.pop_front();
    list = pop_front_result.1;
    assert(pop_front_result.0.id == 1);

    let pop_back_result = list.pop_back();
    list = pop_back_result.0;
    assert(pop_back_result.1.id == 2);

    assert(list.len() == 3);

    let mut remove_result = list.remove(0);
    list = remove_result.0;
    let mut removed_item = remove_result.1;
    assert(removed_item.arr[0] == a);

    remove_result = list.remove(1);
    list = remove_result.0;
    removed_item = remove_result.1;
    assert(removed_item.arr[0] == 0);

    let last_item = list[0];

    assert(last_item.id == b);
    list = list.insert(1, removed_item);

    assert(list.len() == 2);
    assert(list[0].id == b);
    assert(list[1].id == 0);
}
