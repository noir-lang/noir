use dep::std;

mod foo;

struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(x : Field, y : pub myStruct, z: pub foo::bar::barStruct, a: pub foo::fooStruct) -> pub Field {
    let struct_from_bar = foo::bar::barStruct { val: 1, array: [0, 1] };

    check_inner_struct(a, z);

    for i in 0..std::array::len(struct_from_bar.array) {
        constrain struct_from_bar.array[i] == z.array[i];
    }
    constrain z.val == struct_from_bar.val;

    constrain (struct_from_bar.val * x) == x;

    constrain x != y.bar;

    constrain a.bar_struct.message == "hello";
    
    a.bar_struct.array[1]
}

fn check_inner_struct(a: foo::fooStruct, z: foo::bar::barStruct) {
    constrain a.bar_struct.val == z.val;
    for i in 0..std::array::len(a.bar_struct.array) {
        constrain a.bar_struct.array[i] == z.array[i];
    }
}
