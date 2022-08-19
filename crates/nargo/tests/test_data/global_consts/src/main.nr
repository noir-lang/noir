const N = 32;
fn main(a : [32]Field, b : [32]Field) {
     for i in 0..N {
          constrain a != b;
     }
     constrain a != b;
}