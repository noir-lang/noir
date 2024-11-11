fn main() {
    let (first, rest) = split_first([1, 2, 3, 4]);
    assert_eq(first, 1);
    assert_eq(rest, [2, 3, 4]);

    // Type inference works without the type constraints from assert_eq as well
    let _ = split_first([1, 2, 3]);

    let _ = push_multiple([1, 2, 3]);

    test_constant_folding::<10>();
    test_non_constant_folding::<10, 20>();
}

fn split_first<T, let N: u32>(array: [T; N]) -> (T, [T; N - 1]) {
    std::static_assert(N != 0, "split_first called on empty array");
    let mut new_array: [T; N - 1] = std::mem::zeroed();

    for i in 0..N - 1 {
        new_array[i] = array[i + 1];
    }

    (array[0], new_array)
}

fn push<let N: u32>(array: [Field; N], element: Field) -> [Field; N + 1] {
    let mut result: [_; N + 1] = std::mem::zeroed();
    result[array.len()] = element;

    for i in 0..array.len() {
        result[i] = array[i];
    }

    result
}

fn push_multiple<let N: u32>(array: [Field; N]) -> [Field; N + 2] {
    // : [Field; N + 1]
    let array2 = push(array, 4);

    // : [Field; (N + 1) + 1]
    let array3 = push(array2, 5);

    // [Field; (N + 1) + 1] = [Field; N + 2]
    array3
}

// This signature fails because we can't match `_ + 1` to `3` at the call site
// fn push_multiple<let N: u32>(array: [Field; 1 + N]) -> [Field; N + 3] {

// *********************************************
// The rest of this file is setup for demo_proof
// *********************************************

pub struct W<let N: u32> {}

pub struct Equiv<T, TU, U, UT> {
    // TODO(https://github.com/noir-lang/noir/issues/5644):
    // Bug with struct_obj.field_thats_a_fn(x)

    to_: fn[TU](T) -> U,
    fro_: fn[UT](U) -> T,
    // .. other coherence conditions
}

impl<T, TU, U, UT> Equiv<T, TU, U, UT> {
    fn to(self, x: T) -> U {
        (self.to_)(x)
    }

    fn fro(self, x: U) -> T {
        (self.fro_)(x)
    }
}

fn equiv_trans<T, TU, U, UT, UV, V, VU>(
    x: Equiv<T, TU, U, UT>,
    y: Equiv<U, UV, V, VU>,
    ) -> Equiv<T, (Equiv<U, UV, V, VU>, Equiv<T, TU, U, UT>), V, (Equiv<T, TU, U, UT>, Equiv<U, UV, V, VU>)> {
    Equiv { to_: |z| y.to(x.to(z)), fro_: |z| x.fro(y.fro(z)) }
}

fn mul_one_r<let N: u32>() -> Equiv<W<N>, (), W<N>, ()> {
    Equiv { to_: |_x| W {}, fro_: |_x| W {} }
}

fn add_equiv_r<let C: u32, let N: u32, let M: u32, EN, EM>(
    _: Equiv<W<N>, EN, W<M>, EM>,
) -> Equiv<W<C + N>, (), W<C + M>, ()> {
    Equiv { to_: |_x| W {}, fro_: |_x| W {} }
}

fn mul_comm<let N: u32, let M: u32>() -> Equiv<W<N * M>, (), W<M * N>, ()> {
    Equiv { to_: |_x| W {}, fro_: |_x| W {} }
}

fn mul_add<let N: u32, let M: u32, let P: u32>() -> Equiv<W<N * (M + P)>, (), W<N * M + N * P>, ()> {
    Equiv { to_: |_x| W {}, fro_: |_x| W {} }
}

// (N + 1) * N == N * N + N
pub fn demo_proof<let N: u32>() -> Equiv<W<(N * (N + 1))>, (Equiv<W<N>, (), W<N>, ()>, Equiv<W<(N * (N + 1))>, (Equiv<W<(N * (N + 1))>, (), W<N>, ()>, Equiv<W<(N * (N + 1))>, (), W<(N * (N + 1))>, ()>), W<N>, (Equiv<W<(N * (N + 1))>, (), W<(N * (N + 1))>, ()>, Equiv<W<(N * (N + 1))>, (), W<N>, ()>)>), W<N>, (Equiv<W<(N * (N + 1))>, (Equiv<W<(N * (N + 1))>, (), W<N>, ()>, Equiv<W<(N * (N + 1))>, (), W<(N * (N + 1))>, ()>), W<N>, (Equiv<W<(N * (N + 1))>, (), W<(N * (N + 1))>, ()>, Equiv<W<(N * (N + 1))>, (), W<N>, ()>)>, Equiv<W<N>, (), W<N>, ()>)> {
    let p1: Equiv<W<(N + 1) * N>, (), W<N * (N + 1)>, ()> = mul_comm();
    let p2: Equiv<W<N * (N + 1)>, (), W<N * N + N>, ()> = mul_add::<N, N, 1>();
    let p3_sub: Equiv<W<N>, (), W<N>, ()> = mul_one_r();
    let _p3: Equiv<W<N * N + N>, (), W<N * N + N>, ()> = add_equiv_r::<N * N, N, N, _, _>(p3_sub);
    let _p1_to_2 = equiv_trans(p1, p2);

    // equiv_trans(p1_to_2, p3)
    std::mem::zeroed()
}

fn test_constant_folding<let N: u32>() {
    // N + C1 - C2 = N + (C1 - C2)
    let _: W<N + 5 - 2> = W::<N + 3> {};

    // N - C1 + C2 = N - (C1 - C2)
    let _: W<N - 3 + 2> = W::<N - 1> {};

    // N * C1 / C2 = N * (C1 / C2)
    let _: W<N * 10 / 2> = W::<N * 5> {};
    // This case is invalid due to integer division
    // If N does not divide evenly with 10 then we cannot simplify it.
    // e.g. 15 / 10 * 2 = 2 versus 15 / 5 = 3
    //
    // N / C1 * C2 = N / (C1 / C2)
    // let _: W<N / 10 * 2> = W::<N / 5> {};
}

fn test_non_constant_folding<let N: u32, let M: u32>() {
    // N + M - M = N
    let _: W<N + M - M> = W::<N> {};

    // N - M + M = N
    let _: W<N - M + M> = W::<N> {};

    // N * M / M = N
    let _: W<N * M / M> = W::<N> {};
    // This case is not true due to integer division rounding!
    // Consider 5 / 2 * 2 which should equal 4, not 5
    //
    // N / M * M = N
    // let _: W<N / M * M> = W::<N> {};
}
