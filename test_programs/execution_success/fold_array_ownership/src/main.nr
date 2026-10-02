// Each `#[fold]` function is compiled to its own circuit, which owns its array arguments,
// its copy of the globals, and its outputs. Mutable Array Set Optimizations relies on that to
// mark in-place writes, so none of the writes below may be observed through another value.
global G: [Field; 3] = [1, 2, 3];

fn main(
    i: u32,
    v: Field,
    a: [Field; 3],
    ) -> pub ((Field, Field, [Field; 3]), (Field, [Field; 3]), ([Field; 3], [Field; 3]), ([Field; 3], [Field; 3])) {
    (
        callee_writes_argument(a, i), caller_writes_global(i, v),
        fold_returns_one_array_twice(i, v), brillig_returns_one_array_twice(a, i, v),
    )
}

// The callee writes in place into its array parameter, which the caller reads again.
fn callee_writes_argument(a: [Field; 3], i: u32) -> (Field, Field, [Field; 3]) {
    let b = sum_after_write(a, i);
    let c = sum_after_write(a, (i + 1) % 3);
    (b, c, a)
}

// The caller writes in place into a global that the callee reads.
fn caller_writes_global(i: u32, v: Field) -> (Field, [Field; 3]) {
    let mut t = G;
    t[i] = v;
    let r = read_global(i);
    (r, t)
}

// A callee returning one array twice hands back two independent arrays.
fn fold_returns_one_array_twice(i: u32, v: Field) -> ([Field; 3], [Field; 3]) {
    let (mut p, q) = dup_fold([4, 5, 6]);
    p[i] = v;
    (p, q)
}

fn brillig_returns_one_array_twice(a: [Field; 3], i: u32, v: Field) -> ([Field; 3], [Field; 3]) {
    // Safety: test
    let (mut p, q) = unsafe { dup_brillig(a) };
    p[i] = v;
    (p, q)
}

#[fold]
fn sum_after_write(arr: [Field; 3], i: u32) -> Field {
    let mut a = arr;
    a[i] += 5;
    a[0] + a[1] + a[2]
}

#[fold]
fn read_global(i: u32) -> Field {
    G[i]
}

#[fold]
fn dup_fold(m: [Field; 3]) -> ([Field; 3], [Field; 3]) {
    (m, m)
}

unconstrained fn dup_brillig(m: [Field; 3]) -> ([Field; 3], [Field; 3]) {
    (m, m)
}
