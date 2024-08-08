
global ITERATION_COUNT = 100000;

unconstrained fn some_unconstrained_fn(x: Field, y: Field) -> Field {
    let x_u128 = U128::from_integer(x);
    let y_u128 = U128::from_integer(y);
    (x_u128 / y_u128).to_integer()
}

fn main(data: [Field; ITERATION_COUNT]) -> pub Field {
    let mut acc = data[0];

    for i in 1..ITERATION_COUNT {
        acc = some_unconstrained_fn(acc, data[i]);
    }

    acc
}
