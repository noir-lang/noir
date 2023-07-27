use dep::std;

fn main(mut x: Field) {

    let add1 = |z| {
        *z = *z + 1;
    };

    let add2 = |z| {
        *z = *z + 2;
    };

    add1(&mut x);
    assert(x == 1);

    add2(&mut x);
    assert(x == 3);

}
