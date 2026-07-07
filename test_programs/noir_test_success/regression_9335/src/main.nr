global G_B: u64 = 2288862572;
fn main(a: u64) -> pub u64 {
    let b = a * 18259827970668162041;
    println(f"b = {b}");
    assert(b == G_B, "YHV");
    b
}

#[test(should_fail_with = "attempt to multiply with overflow")]
fn fuzz_main(a: u8) -> u64 {
    main((a as u64) + 1866214362)
}
