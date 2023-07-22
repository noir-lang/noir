use dep::std;

type foo<T> = [T; 2];
//type bar = Field;

fn main(x : [Field; 2]) {
    let a: foo<T> = [1, 2];
    assert(a[0] != x[0]);

    //let b: bar = 2;
    //assert(x[0] == b);
}
