type foo<T> = [T;2];

fn main(x : [Field; 2]) {
    let a: foo<Field> = [1, 2];
    // assert(a[0] != x[0]);
}
