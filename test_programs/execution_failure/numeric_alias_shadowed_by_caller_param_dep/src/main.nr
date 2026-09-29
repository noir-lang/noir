// `numeric_alias_lib::Cap` is defined as `MAX_LEN`, the library's global equal to 4.
// The parameter `MAX_LEN` must not be what the alias evaluates to here,
// otherwise the first assertion becomes `MAX_LEN <= MAX_LEN` and is dropped.
fn main(MAX_LEN: pub u32, len: u32) {
    assert(MAX_LEN <= numeric_alias_lib::Cap);
    assert(len <= MAX_LEN);
}
