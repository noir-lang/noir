fn main() {
    let a = [1, 2, 3].as_vector();
    foo(f"A: {a} is not 1!");
    foo(());
    foo(a);
}

fn foo<T>(x: T) {
    assert(false, x);
}
