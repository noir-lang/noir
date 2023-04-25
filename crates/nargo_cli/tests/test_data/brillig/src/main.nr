// Test unsafe integer multiplication with overflow: 12^8 = 429 981 696
// The circuit should handle properly the growth of the bit size

#[oracle(rand)]
fn rand(_a: u32) -> Field {}


fn main(mut x: Field, y: Field) {
    let x32 = x as u32;
    // let a = unsafe_if_loop(x32) as Field;
    let a = 5;
    let b = unsafe_inverse(a);
    constrain a*b == 1;

    bar(x);
    bar2(x32);

    let y32 = y as u32;
    constrain y32 % x32 == unsafe_mod(y32,x32);

    constrain f2() == 5;
    constrain f2_unsafe() == 5;
    constrain compute_nullifier(x != 0, x) == x + 3;
    call([x,y]);
}

unconstrained fn unsafe_inverse(x: Field) ->Field {
    if x == 0 {
        0
    } else {
        1/x
    }
}

unconstrained fn unsafe_if_loop(mut x: u32) ->u32 {
    for _i in 0..3 {
        x = x+2;
        if x == 1 {
            x = x+2;
        }
    }
    x
}


unconstrained fn unsafe_mod(x: u32, y: u32) -> u32 {
  x % y
}

unconstrained fn f1(x: Field) ->[Field;1] {
   [ x + 2]
}


fn bar(x: Field) {
    constrain x != f1(x)[0];
}

fn bar2(x:u32) {
  constrain x & 251 != 0;
}


unconstrained fn get_secret_key(x: Field) -> Field {
    x+3
}

fn compute_nullifier(self :bool, note_hash: Field) -> Field {
    let mut secret_key = 0;

    if(self){
        secret_key = get_secret_key(note_hash);
        let check_public_key = note_hash;
        constrain (check_public_key != 0);
    }

    secret_key
}


fn nest1(x: [Field;2]) -> Field{
   nest2(x) + nest2(x) + 1
}
fn nest2(x: [Field;2]) -> Field{
    x[0]+ x[1]+1
}
unconstrained fn nest1_unsafe(x: [Field;2]) -> Field{
   nest2_unsafe(x) + nest2_unsafe(x) + 1
}
unconstrained fn nest2_unsafe(x: [Field;2]) -> Field{
    x[0]+ x[1]+1
}

unconstrained fn f2()-> Field{
    nest1([0,1])
}

unconstrained fn f2_unsafe()-> Field{
    nest1_unsafe([0,1])
}

unconstrained fn call_internal(
    args: [Field; 2]
) -> [Field; 2] {
   [args[0]+2, 2]
}

fn call(array: [Field;2]) {
    let tyo = call_internal(array);
    constrain tyo[0] == array[0] + tyo[1];
}