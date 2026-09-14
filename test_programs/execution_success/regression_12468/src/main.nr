struct Phantom {}

fn main(c: bool, idx: u32) {
    let a: [[Phantom; 3]] = @[[Phantom {}; 3], [Phantom {}; 3], [Phantom {}; 3]];
    let b: [[Phantom; 3]] = @[[Phantom {}; 3], [Phantom {}; 3]];
    let s = if c { a } else { b };
    let r = s.remove(idx);
    assert(r.0.len() < 5);
}
