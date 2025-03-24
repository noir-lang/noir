struct ArrData<let N: u32> {
    a: [Field; N],
    b: [Field; N + N - 1],
}

fn main() {
    let _f: ArrData<5> = ArrData { a: [0; 5], b: [0; 9] };
}
