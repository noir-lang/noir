global ARRAY_LEN: u32 = 3;

fn main(arr: [Field; ARRAY_LEN], x: u32) -> pub Field {
    let mut value = arr[ARRAY_LEN - 1];

    value += arr[0u32];
    value += arr[1u32];

    value + x as Field
}
