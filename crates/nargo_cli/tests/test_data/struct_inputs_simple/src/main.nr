use dep::std;

struct myStruct {
    foo: u32,
    bar: Field,
}

fn main(y : pub myStruct) {
    constrain y.foo == 5;
    constrain y.bar == 7;
}

