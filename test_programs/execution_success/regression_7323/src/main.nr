// This program lead to panics previously due to the compiler lowering it to multiple blocks
// which all jumped to the same end block. It runs now due to the compiler lowering to the
// equivalent of a nested series of if-else instead.
fn main(x: Field) {
    match x {
        1 => panic("Branch 1 should not be taken"),
        2 => panic("Branch 2 should not be taken"),
        3 => panic("Branch 3 should not be taken"),
        _ => (),
    }
}
