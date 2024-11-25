mod bar;

global N: u32 = 5;
global MAGIC_NUMBER: u32 = 3;
global TYPE_INFERRED: u32 = 42;

pub fn from_foo(x: [Field; bar::N]) {
    for i in 0..bar::N {
        assert(x[i] == bar::N as Field);
    }
}
