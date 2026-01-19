fn main() {
    comptime {
        let z = || assert(false);
        let y = || z();
        let x = || y();
        x()
    }
}
