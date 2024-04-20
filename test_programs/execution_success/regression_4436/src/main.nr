trait LibTrait<N> {
    fn broadcast();
    fn get_constant() -> field;
}

global STRUCT_A_LEN: field = 3;
global STRUCT_B_LEN: field = 5;

struct StructA;
struct StructB;

impl LibTrait<u32> for StructA {
    fn broadcast() {
        Self::get_constant();
    }

    fn get_constant() -> field {
        1
    }
}
impl LibTrait<u64> for StructB {
    fn broadcast() {
        Self::get_constant();
    }

    fn get_constant() -> field {
        1
    }
}

fn main() {}
