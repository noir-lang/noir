fn main(flag: bool, arr: [Field; 4]) {
    let sli = arr.as_slice();
    println(sli);
    std::hint::black_box(arr);
}
