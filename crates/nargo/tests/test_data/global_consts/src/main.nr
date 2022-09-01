mod foo;

let M: const Field = 32;
let L: const Field = 10; // Unused globals currently allowed
let N: const Field = 5;
//let N: const Field = 5;

fn main(a: [M]Field, b: [M]Field) {
     constrain N != M;

     let mut y = 5;
     for i in 0..N*N {
          //let N: const Field = 10;
          y = i;
     };
     constrain y == 24;

     let mut x = M;
     //x = 5;
     constrain x == 32;

     let q = multiplyByM(3);
     constrain q == 96;

     arrays_neq(a, b);

     let arr: [N]Field = [N; 5];
     // let arr: [N]Field = [N; 5]; NOTE: this is currently broken
     constrain arr[0] == 5;
     constrain arr[4] == 5;

     //let t: [2]Field = [N, M];
     // NOTE: This is currently broken, as only global consts work in array annotations
     //let L: const Field = 2;
     let t: []Field = [N, M];

     constrain t[1] == 32;

     mysubmodule::my_helper();
     //constrain 5 == mysubmodule::N;
}

fn multiplyByM(x: Field) -> Field {
     // Uncomment this line to show duplicate variables error
     //let M: const Field = 10;
     x * M
}

fn arrays_neq(a: [M]Field, b: [M]Field) {
     constrain a != b;
}

mod mysubmodule {
    use dep::std;
    // Comment out the line below to show that consts are scoped within a module
    let N: const Field = 10;

    fn my_bool_or(x: Field, y: Field) {
        constrain std::bool_or(x, y) == 1;
    }

    fn my_helper() {
        let mut x = N;
        constrain x == 5;
    }
}
