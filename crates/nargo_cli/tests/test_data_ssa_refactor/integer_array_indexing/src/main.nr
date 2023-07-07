use dep::std;

global ARRAY_LEN: u32 = 3;

fn main(arr: [Field; ARRAY_LEN], x: u32) -> pub Field {

    let mut value = arr[ARRAY_LEN - 1];

    value += arr[0 as u32];
    value += arr[1 as Field];

    value + (x as Field)

}
