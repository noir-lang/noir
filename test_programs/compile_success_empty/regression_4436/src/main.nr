trait LibTrait<N> {
    fn broadcast();
    fn get_constant() -> Field;
}

pub global STRUCT_A_LEN: u32 = 3;
pub global STRUCT_B_LEN: u32 = 5;

pub struct StructA;
pub struct StructB;

impl LibTrait<u32> for StructA {
    fn broadcast() {
        let _ = Self::get_constant();
    }

    fn get_constant() -> Field {
        1
    }
}
impl LibTrait<u64> for StructB {
    fn broadcast() {
        let _ = Self::get_constant();
    }

    fn get_constant() -> Field {
        1
    }
}

fn main() {}
