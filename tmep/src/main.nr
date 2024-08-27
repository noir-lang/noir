fn main(x: [Field; 2]) {
    foo(x)
}

fn foo<T, N>(a: [T; N]) {
    bar(a, |b, c| b < c);
}

fn bar<T, N>(a: [T; N], cmp: fn(T, T) -> bool) {
    let _ = cmp(a[0], a[1]);
}
