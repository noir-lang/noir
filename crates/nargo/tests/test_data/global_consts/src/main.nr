//let N: const Field = 32;
let N: const Field = 5;
fn main(a : [32]Field, b : [32]Field) {
     // let q = [N]Field;
     // let N: const Field = 5;
     let mut x = N;
     for i in 0..5 {
          x = i;
     };
     constrain x == 4;
     
     //let y = N;
     //constrain y == 5;
     
     constrain a != b;
}