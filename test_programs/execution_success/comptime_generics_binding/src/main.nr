fn gen_func<let N: u32>() -> u32 {
    N
}

fn factory<let M: u32>(callback: fn() -> u32) -> (fn() -> u32, u32) {
    let new_fn = gen_func::<M>;
    let cb_result = callback();
    (new_fn, cb_result)
}

fn dummy() -> u32 {
    0
}

fn main() {
    let runtime_val = {
        let (f, _) = factory::<5>(dummy);
        let (_, val) = factory::<99>(f);
        val
    };

    // Not affected by new instantiation (factory(99))
    assert(runtime_val == 5);

    let comptime_val = {
        comptime {
            let (f, _) = factory::<5>(dummy);
            let (_, val) = factory::<99>(f);
            val
        }
    };

    // Comptime must matches runtime
    assert(comptime_val == 5);
}
