unconstrained fn read_double_ref(r: &&Field) -> Field {
    **r
}

unconstrained fn read_array_of_refs(r: [&u32; 2]) -> u32 {
    *r[0] + *r[1]
}

fn main(x: pub Field, y: pub u32) {
    // Safety:
    let result = unsafe { read_double_ref(&&x) };
    assert_eq(result, x);

    // Safety:
    let result = unsafe { read_array_of_refs([&y, &y]) };
    assert_eq(result, y + y);
}
