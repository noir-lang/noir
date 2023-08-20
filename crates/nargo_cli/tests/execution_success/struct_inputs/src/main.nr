use dep::std;

mod foo;

struct myStruct {
    foo: u32,
    bar: Field,
    message: str<5>,
}

fn main(x : Field, y : pub myStruct, z: pub foo::bar::barStruct, a: pub foo::fooStruct) -> pub Field {
    let struct_from_bar = foo::bar::barStruct { val: 1, array: [0, 1], message: "hello" };

    check_inner_struct(a, z);

    for i in 0 .. struct_from_bar.array.len() {
        std::assert_eq(struct_from_bar.array[i], z.array[i]);
    }
    std::assert_eq(z.val, struct_from_bar.val);

    std::assert_eq((struct_from_bar.val * x), x);

    assert(x != y.bar);

    std::assert_eq(y.message, "hello");
    std::assert_eq(a.bar_struct.message, struct_from_bar.message);
    
    a.bar_struct.array[1]
}

fn check_inner_struct(a: foo::fooStruct, z: foo::bar::barStruct) {
    std::assert_eq(a.bar_struct.val, z.val);
    for i in 0.. a.bar_struct.array.len() {
        std::assert_eq(a.bar_struct.array[i], z.array[i]);
    }
}
