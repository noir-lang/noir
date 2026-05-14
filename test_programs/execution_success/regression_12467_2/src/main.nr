struct Phantom {}

fn main(idx: u32) {
    let s: [Phantom] = @[Phantom {}, Phantom {}, Phantom {}];
    let _ = s.remove(idx);
}
