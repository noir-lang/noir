type BigArray = [Field; 4294967295];

#[oracle(void_to_array)]
unconstrained fn void_to_array_oracle() -> BigArray {}

unconstrained fn void_to_array() -> BigArray {
    void_to_array_oracle()
}

unconstrained fn main() -> pub BigArray {
    void_to_array()
}
