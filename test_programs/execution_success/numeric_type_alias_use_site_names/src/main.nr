// A numeric type alias means the same number wherever it is used: in value position it
// evaluates to what it means in type position, whatever names are in scope at the use site.
mod lib {
    pub global MAX_LEN: u32 = 4;
    pub type Cap: u32 = MAX_LEN;

    pub global N: u32 = 2;
    pub type Size: u32 = N;
    pub type Double<let N: u32>: u32 = N * 2;
    pub type Quad<let N: u32>: u32 = Double::<N> + Double::<N>;
}

mod local {
    pub global N: u32 = 2;
    pub type Size: u32 = N;
    pub type Plus<let A: u32>: u32 = A + N;
    pub type Mul<let A: u32, let B: u32>: u32 = A * B;
}

// Composed generic aliases in a module with no item named like the alias generic.
mod no_global {
    pub type Double<let N: u32>: u32 = N * 2;
    pub type Quad<let N: u32>: u32 = Double::<N> + Double::<N>;
}

fn numeric_generic_named_like_global<let N: u32>() -> u32 {
    local::Size
}

fn numeric_generic_named_like_alias_generic<let N: u32>() -> u32 {
    local::Plus::<N>
}

fn turbofish_with_caller_generic<let M: u32>() -> u32 {
    lib::Double::<M>
}

fn composed_with_caller_generic<let M: u32>() -> u32 {
    lib::Quad::<M>
}

fn swapped_generics<let A: u32, let B: u32>() -> u32 {
    local::Mul::<B, A> - A * B + B
}

fn generic_arithmetic_in_turbofish<let K: u32>() -> u32 {
    turbofish_with_caller_generic::<K + 1>()
}

fn value_and_type_position_agree<let M: u32>() -> bool {
    let a: [u8; lib::Quad::<M>] = [0; lib::Quad::<M>];
    a.len() == lib::Quad::<M>
}

fn main(N: pub u32, MAX_LEN: u32, x: u32) {
    // Parameters named like the globals in the alias bodies.
    assert_eq(local::Size, 2);
    assert_eq(lib::Size, 2);
    assert_eq(lib::Cap, 4);
    assert(MAX_LEN <= lib::Cap);
    assert(N != lib::Size);

    // Numeric generics named like the global, and like the alias's own generic.
    assert_eq(numeric_generic_named_like_global::<9>(), 2);
    assert_eq(numeric_generic_named_like_alias_generic::<10>(), 12);

    // Closure parameters.
    let f = |N: u32| local::Size + N;
    assert_eq(f(x), 2 + x);

    // `let` bindings.
    {
        let N = x + 1;
        let MAX_LEN = x + 2;
        assert_eq(local::Size, 2);
        assert_eq(lib::Cap, 4);
        assert_eq(N, x + 1);
        assert_eq(MAX_LEN, x + 2);
    }

    // Generic aliases.
    assert_eq(turbofish_with_caller_generic::<5>(), 10);
    assert_eq(composed_with_caller_generic::<3>(), 12);
    assert_eq(swapped_generics::<3, 7>(), 7);
    assert_eq(generic_arithmetic_in_turbofish::<4>(), 10);
    assert(value_and_type_position_agree::<5>());

    // Composed generic aliases, with and without an item named like the alias generic.
    assert_eq(lib::Quad::<3>, 12);
    let a: [u8; lib::Quad::<3>] = [0; 12];
    assert_eq(a.len(), lib::Quad::<3>);
    assert_eq(no_global::Quad::<3>, 12);

    // The comptime interpreter agrees.
    assert_eq(comptime { lib::Quad::<3> }, 12);

    // Loop bounds.
    let mut sum = 0;
    for i in 0..lib::Double::<3> {
        sum += i;
    }
    assert_eq(sum, 15);
}
