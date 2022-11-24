global N = 100;

fn main (x: [u8;5], y:[Field;2]) -> pub [Field; N]
{
    let mut r = [0;N];
    for i in 0..N {
        let id = i as u32 /5;
        let j = i -5*id as Field;
        let ic = i as u32/2;
        let k = i-2*ic as Field;
        r[i] = x[j] as Field  + y[k];
    };
    r
}
    