use dep::std;

fn main(x: Field, y: Field) {
    let pair = (x, y);
    assert(pair.0 == 1);
    assert(pair.1 == 0);

    let (a, b) = if true { (0, 1) } else { (2, 3) };
    assert(a == 0);
    assert(b == 1);

    let (u,v) = if x as u32 <1 {
        (x,x+1)
    } else {
        (x+1,x)
    };
   assert(u==x+1);
   assert(v==x);
}
