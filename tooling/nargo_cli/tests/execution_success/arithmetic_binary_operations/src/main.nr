// Tests a very simple program.
// 
// The features being tested are:
// Binary addition, multiplication, division, constant modulo
// x = 3, y = 4, z = 5

struct V { q: u32, r: u32 }
struct U { v: V, w: (u32,u32) }

fn main(x : Field, y : Field, z : Field) -> pub Field {
    let uu = U { v: V { q: 100, r: 200 }, w: (300,400) };
    //let U { v: V { q, ..}, w: (w1,w1) } = uu;
    let U { v, w } = uu;
    //let (q,r,(s,t)) = (3,4,(5,6));

    //cast
    //assert(y as u1 == 0);

    let a = x + x; // 3 + 3 = 6
    let b = a - y; // 7 - 4 = 2
    let c = b * z; // 2 * 5 = 10
    let d = c / a; // 10 / 6 (This uses field inversion, so we test it by multiplying by `a`)
    d * a
}
