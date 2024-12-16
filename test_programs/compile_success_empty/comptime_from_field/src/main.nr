fn main() {
    comptime {
        let _: Field = U128::from_hex("0x0").to_integer();
    }
}

