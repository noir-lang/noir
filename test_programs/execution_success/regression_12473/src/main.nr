struct Phantom {}

fn main(c: bool) {
    let a: [Phantom] = @[Phantom {}, Phantom {}, Phantom {}];
    let b: [Phantom] = @[Phantom {}, Phantom {}];
    let s = if c { a } else { b };
    let r = s.pop_back();
    assert(r.0.len() < 5);
}
