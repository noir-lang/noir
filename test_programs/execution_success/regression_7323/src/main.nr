// This program lead to panics previously due to the compiler lowering it to multiple blocks
// which all jumped to the same end block. It runs now due to the compiler lowering to the
// equivalent of a nested series of if-else instead.
fn main(x: Field) {
    match x {
        1 => println(1),
        2 => println(2),
        3 => println(3),
        x => println(x),
    }
}
