fn main() {
    comptime {
        let x = 4;
        let y = 5;
        println(f"x is {x}, fake interpolation: \{y}, y is {y}");
        println(std::unsafe::zeroed::<fmtstr<4, ()>>());
    }
}
