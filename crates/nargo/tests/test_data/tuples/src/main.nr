use dep::std;

fn main(x: Field, y: Field) {
    let pair = (x, y);
    constrain pair.0 == 1;
    constrain pair.1 == 0;

    let (a, b) = if true { (0, 1) } else { (2, 3) };
    constrain a == 0;
    constrain b == 1;

   let cc = if x as u32 <1 { [0,1] } else { [2,3] };
   constrain cc[0] == 2;

    let (u,v) = if x as u32 <1 {
        (x,x+1)
    } else {
        (x+1,x)
    };
   constrain u==x+1;
   constrain v==x;
}
