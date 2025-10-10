pub fn main() {
    println(bar);
    println(std::as_witness);
}

#[oracle(barnacle)]
unconstrained fn bar() -> Field {}
