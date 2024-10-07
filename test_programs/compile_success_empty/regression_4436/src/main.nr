trait LibTrait<N> {
    fn broadcast();
    fn get_constant() -> Field;
}

global STRUCT_A_LEN: u32 = 3;
global STRUCT_B_LEN: u32 = 5;

struct StructA;
struct StructB;

impl LibTrait<u32> for StructA {
    fn broadcast() {
        Self::get_constant();
    }

    fn get_constant() -> Field {
        1
    }
}
impl LibTrait<u64> for StructB {
    fn broadcast() {
        Self::get_constant();
    }

    fn get_constant() -> Field {
        1
    }
}

fn main() {}
