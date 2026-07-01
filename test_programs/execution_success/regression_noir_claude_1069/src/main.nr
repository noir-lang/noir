fn main(arr: [Field; 3]) {
    let c = || {
        let mut a = arr;
        a[0] = 10;
        let _ = a;
        call(a);
        arr
    };
    let output = c();
    assert_eq(output, [1, 2, 3]);
}

fn call<T>(_: T) {}
