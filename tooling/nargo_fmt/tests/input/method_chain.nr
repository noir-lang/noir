fn main(a: Field, b: Field, c: Field, d: Field, e: Field) {
    let one_line = a.pow_32(b);
    let out = a.pow_32(b).pow_32(c).pow_32(d);
    assert(out == e);
}

// fn main(a: Field, b: Field, c: Field, d: Field, e: Field, f: Field, g: Field, h: Field) {
//     let out = a.pow_32(b).pow_32(c).pow_32(d).pow_32(e).pow_32(f).pow_32(g).pow_32(h).pow_32(i).pow_32(j);
//     assert(out == k);
// }
//
// fn main(a: Field, b: Field, c: Field, d: Field, e: Field, f: Field, g: Field, h: Field) {
//     a.pow_32(b)
//      .pow_32(c)
//      .pow_32(d)
//      .pow_32(e)
//      .pow_32(f)
//      .pow_32(g)
//      .pow_32(h)
//      .pow_32(i)
//      .pow_32(j);
//     assert(a == k);
// }
