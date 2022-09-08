mod foo;

const M: Field = 32;
const L: Field = 10; // Unused globals currently allowed
const N: const Field = 5;
//let N: const Field = 5; // Uncomment to see duplicate globals error 

fn main(a: [M]Field, b: [M]Field) {
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

     let N_local: const Field = 10;
     let arr: [N_local]Field = [N; 10];
     constrain arr[0] == 5;
     constrain arr[9] == 5;

     //let mut L: Field = 2; // Uncomment to show expected const error for array annotations
     let L: const Field = 2;
     let t: [L]Field = [N, M];

     constrain t[1] == 32;

     constrain 15 == mysubmodule::my_helper();
     constrain 10 == mysubmodule::N;
     
     let add_submodules_N = mysubmodule::N + foo::N;
     constrain 15 == add_submodules_N;
}

fn multiplyByM(x: Field) -> Field {
     x * M
}

fn arrays_neq(a: [M]Field, b: [M]Field) {
     constrain a != b;
}

mod mysubmodule {
    use dep::std;

    const N: Field = 10;
    const L: Field = 50;

    fn my_bool_or(x: Field, y: Field) {
        constrain std::bool_or(x, y) == 1;
    }

    fn my_helper() -> const Field {
        let N: const Field = 15; // Like in Rust, the local variable overrides the global const
        let mut x = N;
        x
    }
}
