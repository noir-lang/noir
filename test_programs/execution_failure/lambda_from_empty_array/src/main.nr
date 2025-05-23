fn main() {
    // Still panics when there are no other lambdas
    // This should fail either way as we are attempting to access an empty array at zero
    let lambdas: [fn(()) -> (); 0] = [];
    lambdas[0](());
}
