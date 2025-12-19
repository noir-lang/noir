use std::vector;
// Tests nested vector passing to/from functions
unconstrained fn push_back_to_vector<T>(vector: [T], item: T) -> [T] {
    vector.push_back(item)
}

struct NestedVectorStruct {
    id: Field,
    arr: [Field]
}

unconstrained fn create_foo(id: Field, value: Field) -> NestedVectorStruct {
    let mut arr = [id];
    arr = arr.push_back(value);
    NestedVectorStruct { id, arr }
}

unconstrained fn main(a: Field, b: Field) {
    let mut vector = [create_foo(a, b), create_foo(b, a)];
    assert(vector.len() == 2);

    assert(vector[0].id == a);
    assert(vector[0].arr[0] == a);
    assert(vector[1].id == b);
    assert(vector[1].arr[1] == a);

    vector = push_back_to_vector(vector, create_foo(0, 42));
    assert(vector.len() == 3);

    assert(vector[0].id == a);
    assert(vector[0].arr[0] == a);
    assert(vector[1].id == b);
    assert(vector[1].arr[1] == a);

    assert(vector[2].id == 0);
    assert(vector[2].arr[0] == 0);
    assert(vector[2].arr[1] == 42);

    vector = vector.push_front(create_foo(1, 43));
    vector = vector.push_back(create_foo(2, 44));

    assert(vector.len() == 5);

    let pop_front_result = vector.pop_front();
    vector = pop_front_result.1;
    assert(pop_front_result.0.id == 1);

    let pop_back_result = vector.pop_back();
    vector = pop_back_result.0;
    assert(pop_back_result.1.id == 2);

    assert(vector.len() == 3);

    let mut remove_result = vector.remove(0);
    vector = remove_result.0;
    let mut removed_item = remove_result.1;
    assert(removed_item.arr[0] == a);

    remove_result = vector.remove(1);
    vector = remove_result.0;
    removed_item = remove_result.1;
    assert(removed_item.arr[0] == 0);

    let last_item = vector[0];

    assert(last_item.id == b);
    vector = vector.insert(1, removed_item);

    assert(vector.len() == 2);
    assert(vector[0].id == b);
    assert(vector[1].id == 0);
}
