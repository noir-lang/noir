fn dummy() -> u32 {
    0
}

fn no_op() {}

fn create_or_call<let N: u32>(should_call: bool, f: fn() -> u32) -> u32 {
    if should_call {
        f()
    } else {
        let closure = || {
            no_op();
            N
        };
        create_or_call::<99>(true, closure)
    }
}

fn main() {
    let result = create_or_call::<3>(false, dummy);
    assert(result == 3);
}
