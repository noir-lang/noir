use std::mem::checked_transmute;

fn main() {
    comptime {
        let x: Field = 1;
        let _: Field = checked_transmute(x);
    }
}
