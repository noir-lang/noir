global G_A: [Field] = #[-282608142060723387702497189224219065418];
fn main() -> pub Field {
    let (a, _) = G_A.pop_back();
    let (_, f) = a.pop_back();
    f
}
