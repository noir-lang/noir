fn main() {
    comptime {
        let x: u64 = 1;
        let y = x as Field;
        let _ = y as u128;
    }
}
