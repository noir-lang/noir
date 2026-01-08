fn main(val: str<3>, flag: bool, val2: str<1>) {
    std::hint::black_box(());
    example2(val, flag);
    example3(val2);

    // Still broken. See https://github.com/noir-lang/noir/issues/11134
    // example4(val, flag);
}

fn example2(val: str<3>, flag: bool) {
    let b = val.as_bytes();
    let s = b.as_vector();
    let (_, s1) = s.pop_front();
    let (s2, _) = s.pop_back();
    let (s3, s4, _) = std::hint::black_box((s1, s2, 1));
    let _ = if flag { s3 } else { s4 };
}

fn example3(val: str<1>) {
    let b = val.as_bytes();
    let s = b.as_vector();
    let _ = std::hint::black_box((1, s));
}

// fn example4(val: str<3>, flag: bool) {
//     let b = val.as_bytes();
//     let s = b.as_vector();
//     let (_, s1) = s.pop_front();
//     let a = std::hint::black_box(|| {
//         s1
//     });
//     let a = if flag {
//         a()
//     } else {
//         &[123]
//     };
//     println(a);
// }
