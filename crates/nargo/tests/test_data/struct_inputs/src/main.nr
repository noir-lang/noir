mod foo;

struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(x : Field, y : pub myStruct, z: pub foo::bar::barStruct) {
    let struct_from_bar = foo::bar::barStruct { val: 1 };
    // constrain z.val != struct_from_bar.array[0];
    constrain z.val == struct_from_bar.val;

    constrain (struct_from_bar.val * x) == x;

    constrain x != y.bar;
}