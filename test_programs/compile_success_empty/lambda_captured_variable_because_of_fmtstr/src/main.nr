fn main() {
    comptime {
        let captured = quote { };
        let f = || { let _ = f"{captured}"; };
        f();
    }
}
