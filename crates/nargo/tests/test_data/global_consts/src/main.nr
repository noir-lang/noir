mod foo;

const M: Field = 32;
const L: Field = 10; // Unused globals currently allowed
const N: Field = 5;
//const N: Field = 5; // Uncomment to see duplicate globals error 

fn main(a: [Field; M], b: [Field; M], c : pub [Field; foo::MAGIC_NUMBER]) {
     for i in 0..foo::MAGIC_NUMBER {
          constrain c[i] == foo::MAGIC_NUMBER;
     };

     constrain N != M;

     let mut y = 5;
     let mut x = M;
     for i in 0..N*N {
          let M: const Field = 10;
          x = M;
          
          y = i;
     };
     constrain y == 24;
     constrain x == 10;

     let q = multiplyByM(3);
     constrain q == 96;

     arrays_neq(a, b);

     //let mut L: Field = 2; // Uncomment to show expected const error for array annotations
     let L: const Field = 2;
     let t: [Field; L] = [N, M];

     constrain t[1] == 32;

     constrain 15 == mysubmodule::my_helper();
     constrain 10 == mysubmodule::N;
     
     let add_submodules_N = mysubmodule::N + foo::N;
     constrain 15 == add_submodules_N;

     let arr: [Field; mysubmodule::N] = [N; 10];
     constrain arr[0] == 5;
     constrain arr[9] == 5;
     
     // Example showing an array filled with (mysubmodule::N + 2) 0's
     let sugared = [0; mysubmodule::N + 2];
     constrain sugared[mysubmodule::N + 1] == 0;
}

fn multiplyByM(x: Field) -> Field {
     x * M
}

fn arrays_neq(a: [Field; M], b: [Field; M]) {
     constrain a != b;
}

mod mysubmodule {
    use dep::std;

    const N: Field = 10;
    const L: Field = 50;

    fn my_bool_or(x: u1, y: u1) {
        constrain x | y == 1;
    }

    fn my_helper() -> const Field {
        let N: const Field = 15; // Like in Rust, the local variable overrides the global const
        let x = N;
        x
    }
}
