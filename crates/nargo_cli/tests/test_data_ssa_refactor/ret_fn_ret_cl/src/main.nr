use dep::std;

fn f(x: Field) -> Field {
    x
}

fn ret_fn() -> fn(Field) -> Field {
    let y = 1;
    let inner_closure = |z| -> Field{
        z + y
    };
    std::println(inner_closure(1));
    f
}

fn ret_closure() -> fn(Field) -> Field {
    let cl = |z: Field| -> Field {
        z
    };
    cl
}

fn main(x : Field) {
    let result_fn = ret_fn();
    assert(result_fn(x) == x); // Works

    let result_cl = ret_closure();
    assert(result_cl(x) == x);
}
