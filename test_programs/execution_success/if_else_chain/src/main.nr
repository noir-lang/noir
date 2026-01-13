// fn main(a: u32, mut c: [u32; 4]) {
//     if a == c[0] {
//         assert(c[0] == 0);
//     } else if a == c[1] {
//         assert(c[1] == 0);
//     } else if a == c[2] {
//         assert(c[2] == 0);
//     } else if a == c[3] {
//         // expect to match this case
//         assert(c[3] == 0);
//     } else {
//         assert(c[0] == 10);
//     }
// }

// fn main() {
//     // 4294967295u32
//     // 200 million elements - should hit OOM
//     let _arr: [Field; 500000000] = [0; 500000000];
// }

fn wrap<T>(x: T) -> (T, T) {
    (x, x)
}
fn main() {
    let v0 = 1;
    let v1 = wrap(v0);
    let v2 = wrap(v1);
    let v3 = wrap(v2);
    let v4 = wrap(v3);
    // let v5 = wrap(v4);
    // let v6 = wrap(v5);
    // let v7 = wrap(v6);
    // let v8 = wrap(v7);
    // let v9 = wrap(v8);
    // let v10 = wrap(v9);
    // let v11 = wrap(v10);
    // let v12 = wrap(v11);
    // let v13 = wrap(v12);
    // let v14 = wrap(v13);
    // let v15 = wrap(v14);
    // let v16 = wrap(v15);
    // let v17 = wrap(v16);
    // let v18 = wrap(v17);
    // let v19 = wrap(v18);
    // let v20 = wrap(v19);
    // let v21 = wrap(v20);
    // let v22 = wrap(v21);
    // let v23 = wrap(v22);
    // let v24 = wrap(v23);
}
