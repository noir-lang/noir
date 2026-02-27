use std::test::OracleMock;

type BigArray = [Field; 4294967294];

#[oracle(void_to_array)]
unconstrained fn void_to_array_oracle() -> BigArray {}

unconstrained fn void_to_array() -> BigArray {
    void_to_array_oracle()
}

fn main() -> pub BigArray {
    unsafe {
        void_to_array()
    }
}
