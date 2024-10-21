struct MyStruct {
    x: u32,
    y: u32,
    z: u32,
    nested_struct: InnerStruct,
}

struct InnerStruct {
    small_array: [u32; 2],
    big_array: [u32; 5],
}

struct ParentStruct {
    basic_array: [Field; 3],
    id: u32,
    my_structs: [MyStruct; 2],
}

fn main(x: u32, y: pub u32) {
    let nested_struct = InnerStruct { small_array: [1 as u32; 2], big_array: [0 as u32; 5] };
    let s = MyStruct { x, y, z: x + y, nested_struct };
    let parent = ParentStruct { basic_array: [1; 3], id: 100, my_structs: [s, s] };
    let new_parent = map_fields(parent);

    // Now check that the outputs are as we expect them to be
    assert(new_parent.basic_array[0] == 1);
    assert(new_parent.basic_array[1] == 18);
    assert(new_parent.basic_array[2] == 1);

    let struct_0 = new_parent.my_structs[0];
    assert(struct_0.x == 5);
    assert(struct_0.y == 3);
    assert(struct_0.z == 8);
    assert(struct_0.nested_struct.small_array == nested_struct.small_array);
    assert(struct_0.nested_struct.big_array == nested_struct.big_array);

    let struct_1 = new_parent.my_structs[1];
    assert(struct_1.x == 50);
    assert(struct_1.y == 30);
    assert(struct_1.z == 80);
    assert(struct_1.nested_struct.small_array == [5, 10]);
    assert(struct_1.nested_struct.big_array == [15, 20, 25, 30, 35]);
}

// Meaningless mapping to test whether the values returned are what we expect
#[fold]
fn map_fields(mut input: ParentStruct) -> ParentStruct {
    let current_struct = input.my_structs[0];
    let mut sum = 0;
    for value in current_struct.nested_struct.small_array {
        sum += value;
    }
    for value in current_struct.nested_struct.big_array {
        sum += value;
    }
    sum += (current_struct.x + current_struct.y + current_struct.z);

    input.basic_array[1] = sum as Field;

    input.my_structs[1].nested_struct.small_array = [5, 10];
    input.my_structs[1].nested_struct.big_array = [15, 20, 25, 30, 35];

    // LHS input.my_structs[1].x == 50
    input.my_structs[1].x = input.my_structs[1].x * 10;
    // LHS input.my_structs[1].y == 30
    input.my_structs[1].y = input.my_structs[1].y * 10;
    // LHS input.my_structs[1].x == 80
    input.my_structs[1].z = input.my_structs[1].x + input.my_structs[1].y;

    input
}

