#[oracle(foobar)]
unconstrained fn oracle() {}

fn main() {
    // Safety: Test program
    unsafe {oracle()}
}
