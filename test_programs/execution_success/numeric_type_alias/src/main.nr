type Double<let N: u32>: u32 = N * 2;

global N: u32 = 100;
type AliasN<let N: u32>: u32 = N;

fn main(x: Field) {
    // Test that turbofish on a numeric alias resolves to the turbofish value,
    // not to the global N that shares the same name.
    let a: u32 = AliasN::<1>;
    assert(a == 1);

    // Test that N still refers to the global after using AliasN
    assert(N == 100);

    // Test multiple turbofish usages with different values
    let b: u32 = AliasN::<5>;
    assert(b == 5);

    let mut m = new_matrix::<3, 4>();
    m.elements[3 * 2 + 3] = 10;
    m.elements[3 * 2 + x as u32] = 5 + x;
    assert(equal(m, m));
    let b = transpose(m);
    assert(b.elements != m.elements);
    assert(trace(b) != trace(m));
    assert(sum(b) == sum(m));

    let a = test_2::<4>(x);
    assert(a.len() == 1);
}

fn test_2<let N: u32>(x: Field) -> BoundedVec<Field, Double<N>> {
    let mut a = BoundedVec::new();
    a.push(x);
    a
}

type MSize<let N: u32, let M: u32>: u32 = N * M;

struct Matrix<let N: u32, let M: u32> {
    elements: [Field; MSize::<N, M>],
}

fn new_matrix<let N: u32, let M: u32>() -> Matrix<N, M> {
    let mut a = [0; MSize::<N, M>];
    Matrix { elements: a }
}

fn trace<let N: u32, let M: u32>(A: Matrix<N, M>) -> Field {
    let n = if N > M { M } else { N };
    let mut s = 0;
    for i in 0..n {
        s += A.elements[i * N + i];
    }
    s
}

fn sum<let N: u32, let M: u32>(A: Matrix<N, M>) -> Field {
    let mut s = 0;
    for i in 0..MSize::<N, M> {
        s += A.elements[i];
    }
    s
}

fn equal<let N: u32, let M: u32>(A: Matrix<N, M>, B: Matrix<N, M>) -> bool {
    let mut s = true;
    for i in 0..MSize::<N, M> {
        s = s | (A.elements[i] == B.elements[i]);
    }
    s
}

fn transpose<let N: u32, let M: u32>(a: Matrix<N, M>) -> Matrix<M, N> {
    let mut b = new_matrix::<M, N>();
    for i in 0..N {
        for j in 0..M {
            b.elements[j * N + i] = a.elements[i * M + j];
        }
    }
    b
}
