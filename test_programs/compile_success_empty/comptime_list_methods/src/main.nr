fn main() {
    comptime {
        // Comptime scopes are counted as unconstrained
        assert(std::runtime::is_unconstrained());

        let list = &[2];
        let list = list.push_back(3);
        let list = list.push_front(1);
        assert_eq(list, &[1, 2, 3]);

        let list = list.insert(1, 10);
        assert_eq(list, &[1, 10, 2, 3]);

        let (list, ten) = list.remove(1);
        assert_eq(list, &[1, 2, 3]);
        assert_eq(ten, 10);

        let (one, list) = list.pop_front();
        assert_eq(list, &[2, 3]);
        assert_eq(one, 1);

        let (list, three) = list.pop_back();
        assert_eq(list, &[2]);
        assert_eq(three, 3);
    }
}
