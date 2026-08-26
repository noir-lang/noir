// The result of an array read at a statically-safe index must not be multiplied by the
// side-effects predicate: such a read is never redirected, so it needs no repair, and the
// predicate live at that point can belong to the *other* branch — `remove_enable_side_effects`
// is entitled to defer or drop `enable_side_effects` around instructions that declare they
// don't consume it. Masking `accts[0].tag` (read in the `else` arm) by the stale `then`
// condition computes `!cond * cond * tag[i]`, which is identically zero: the read is
// annihilated and any `else`-arm check a zero satisfies is deleted from the circuit.
struct Account {
    balance: Field,
    tag: [u8; 2],
}

fn main(mut accts: [Account; 3], idx: u32, w: Account, cond: bool, y: u32) -> pub [u8; 2] {
    accts[idx] = w;
    let mut r: [u8; 2] = [0, 0];
    if cond {
        r = [(y + 1) as u8, 0];
    } else {
        r = accts[0].tag;
    }
    r
}
