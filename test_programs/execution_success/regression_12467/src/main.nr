struct Phantom {}

fn main(idx: u32) -> pub u32 {
    let s: [Phantom] = @[Phantom {}, Phantom {}, Phantom {}];
    let s2 = s.insert(idx, Phantom {});
    s2.len() as u32
}
