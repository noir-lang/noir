let N: const Field = 5;
let M: const Field = 32;
//a: [M]Field, b: [M]Field
fn main(a: [M]Field, b: [M]Field) {
     // let q = [N]Field;
     // let N: const Field = 5;

     let mut x = N;
     //x = 5;
     constrain x == 5;

     let mut y = N;
     for i in 0..N*N {
          y = i;
     };
     constrain y == 24;

     let q = multiplyByM(3);
     constrain q == 96;

     constrain a != b;
     //let L: const Field = 10;

     let arr: [N]Field = [N; 5];
     constrain arr[0] == 5;

     let t: [2]Field = [N, M];
     constrain t[1] == 32;
}

fn multiplyByM(x: u32) -> u32 {
    //let N: const Field = 5;
    x * M as u32
}
