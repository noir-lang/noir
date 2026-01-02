fn main() {
    comptime {
        // Comptime scopes are counted as unconstrained
        assert(std::runtime::is_unconstrained());

        let vector = &[2];
        let vector = vector.push_back(3);
        let vector = vector.push_front(1);
        assert_eq(vector, &[1, 2, 3]);

        let vector = vector.insert(1, 10);
        assert_eq(vector, &[1, 10, 2, 3]);

        let (vector, ten) = vector.remove(1);
        assert_eq(vector, &[1, 2, 3]);
        assert_eq(ten, 10);

        let (one, vector) = vector.pop_front();
        assert_eq(vector, &[2, 3]);
        assert_eq(one, 1);

        let (vector, three) = vector.pop_back();
        assert_eq(vector, &[2]);
        assert_eq(three, 3);
    }
}
