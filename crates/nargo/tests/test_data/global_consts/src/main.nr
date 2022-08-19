let N: const Field = 5;
fn main(a : [32]Field, b : [32]Field) {
     //let q = [N]Field;
     //let N: const Field = 5;
     let mut x = 0;
     for i in 0..N {
          x = i;
     };
     constrain x == 4;

     constrain a != b;
}