// Regression test for noir-claude#1124:
// `add_to_data_bus` used `element_size` as the SSA-index stride for sub-arrays of
// call_data, emitting type-mismatched/OOB `array_get` ops and ICEing ACIR-gen for
// call_data arrays whose elements contain a nested array of multi-field structs.
struct Pair {
    a: u32,
    b: u32,
}

struct Wrapper {
    x: u32,
    inner: [Pair; 2],
}

fn main(arr: call_data(0) [Wrapper; 3], zero: u32, one: u32, two: u32) -> pub u32 {
    assert_eq(arr[zero].x, 1);
    assert_eq(arr[one].x, 2);
    assert_eq(arr[two].x, 3);

    assert_eq(arr[zero].inner[zero].a, 10);
    assert_eq(arr[zero].inner[zero].b, 11);
    assert_eq(arr[zero].inner[one].a, 12);
    assert_eq(arr[zero].inner[one].b, 13);

    assert_eq(arr[one].inner[zero].a, 20);
    assert_eq(arr[one].inner[zero].b, 21);
    assert_eq(arr[one].inner[one].a, 22);
    assert_eq(arr[one].inner[one].b, 23);

    assert_eq(arr[two].inner[zero].a, 30);
    assert_eq(arr[two].inner[zero].b, 31);
    assert_eq(arr[two].inner[one].a, 32);
    assert_eq(arr[two].inner[one].b, 33);

    arr[two].inner[one].b
}
