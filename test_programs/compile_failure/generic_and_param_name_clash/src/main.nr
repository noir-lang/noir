global N: u32 = 10;

fn hi<let N: u32>(N: u32) {
    // It's not obvious which N would be printed here.
    println(N);
}

fn bye<N>(N: N) {
    // N can only be the variable, the others are types.
    println(N);
}

fn main(N: u32) {
    // The N in the turbofish can only the the global N, because it has to be statically known.
    // The N in the parentheses is the local N, which shadows the global, which is expected.
    hi::<N>(N);
    bye(N);
}
