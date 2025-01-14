fn main() {
    comptime {
        // Comptime scopes are counted as unconstrained
        assert(std::runtime::is_unconstrained());

        let slice = &[2];
        let slice = slice.push_back(3);
        let slice = slice.push_front(1);
        assert_eq(slice, &[1, 2, 3]);

        let slice = slice.insert(1, 10);
        assert_eq(slice, &[1, 10, 2, 3]);

        let (slice, ten) = slice.remove(1);
        assert_eq(slice, &[1, 2, 3]);
        assert_eq(ten, 10);

        let (one, slice) = slice.pop_front();
        assert_eq(slice, &[2, 3]);
        assert_eq(one, 1);

        let (slice, three) = slice.pop_back();
        assert_eq(slice, &[2]);
        assert_eq(three, 3);
    }
}
