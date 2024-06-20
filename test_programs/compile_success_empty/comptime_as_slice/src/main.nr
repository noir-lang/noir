fn main() {
    comptime {
        let ws: [Field; 3] = [1; 3];
        let ws_as_slice: [Field] = ws.as_slice();

        assert_eq(ws[0], ws_as_slice[0]);
    }
}
