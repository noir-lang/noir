struct Phantom {}

fn main() {
    let s: [Phantom] = @[Phantom {}];
    let (s2, _removed) = s.remove(0);
    assert(s2.len() == 0);
}
