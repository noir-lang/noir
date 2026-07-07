fn main() {
    comptime {
        let ws: [Field; 3] = [1; 3];
        let ws_as_vector: [Field] = ws.as_vector();

        assert_eq(ws[0], ws_as_vector[0]);
    }
}
