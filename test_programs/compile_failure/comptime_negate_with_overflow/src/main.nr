fn main() {
    comptime {
        let i: i8 = -(128 as i8);
    }

    comptime {
        let i: u8 = 1;
        let _ = -i;
    }
}
