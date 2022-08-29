let N: const Field = 5;
let M: const Field = 32;
//a: [M]Field, b: [M]Field
fn main(a: [M]Field, b: [M]Field) {
     // let q = [N]Field;
     // let N: const Field = 5;

     //let mut y = 5;
     //for i in 0..N*N {
     //     y = i;
     //};
     //constrain y == 24;
     constrain N != M;

     let mut x = M / 4;
     //x = 5;
     constrain (x) == 8;

     //let q = multiplyByM(3);
     //constrain q == 96;

     //arrays_neq(a, b);
     constrain a != b;

     let arr: [N]Field = [N; 5];
     // let arr: [N]Field = [N; 5]; NOTE: this is currently broken
     constrain arr[0] == 5;
     constrain arr[4] == 5;

     let t: [2]Field = [N, M];
     // NOTE: This is currently broken, as only global consts work in array annotations
     // let L: const Field = 2;
     // let t: [L]Field = [N, M];

     constrain t[1] == 32;
}

fn multiplyByM(x: Field) -> Field {
     // Uncomment this line to show duplicate variables error with global constant 
     //let N: const Field = 5;
     x * M
}

fn arrays_neq(a: [M]Field, b: [M]Field) {
     constrain a != b;
}
