fn make<let N: u32>() -> (fn() -> fn() -> u32, fn(fn() -> u32) -> u32) {
    (
        || || N, // creates a nested closure that returns N
        |f: fn() -> u32| f(), // invokes a given function
    )
}

fn main() {
    // inner_3 should return 3
    let pair_3 = make::<3>();
    let maker_3 = pair_3.0;
    let inner_3 = maker_3();

    let pair_5 = make::<5>();
    let caller_5 = pair_5.1;

    // Call inner_3 through caller_5
    let result = caller_5(inner_3);

    assert_eq(result, 3);
}
