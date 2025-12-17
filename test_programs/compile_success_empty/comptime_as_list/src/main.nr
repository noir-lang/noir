fn main() {
    comptime {
        let ws: [Field; 3] = [1; 3];
        let ws_as_list: [Field] = ws.as_list();

        assert_eq(ws[0], ws_as_list[0]);
    }
}
