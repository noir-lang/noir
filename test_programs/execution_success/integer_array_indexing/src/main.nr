global ARRAY_LEN: u32 = 3;

fn main(arr: [field; ARRAY_LEN], x: u32) -> pub field {
    let mut value = arr[ARRAY_LEN - 1];

    value += arr[0 as u32];
    value += arr[1 as field];

    value + x as field
}
