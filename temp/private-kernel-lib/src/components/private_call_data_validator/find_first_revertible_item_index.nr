use dep::types::{abis::side_effect::Ordered, traits::Empty, utils::arrays::array_length};

pub unconstrained fn find_first_revertible_item_index<T, let N: u32>(
    min_revertible_side_effect_counter: u32,
    items: [T; N],
) -> u32
where
    T: Ordered + Empty + Eq,
{
    let mut index = N;
    for i in 0..N {
        let item = items[i];
        if (index == N) & (item.counter() >= min_revertible_side_effect_counter) {
            index = i;
        }
    }
    if index == N {
        index = array_length(items);
    }
    index
}

mod tests {
    use crate::components::private_call_data_validator::find_first_revertible_item_index::find_first_revertible_item_index;
    use dep::types::tests::fixture_builder::FixtureBuilder;

    struct TestBuilder {
        private_call: FixtureBuilder,
    }

    impl TestBuilder {
        pub fn new() -> Self {
            let private_call = FixtureBuilder::new();
            TestBuilder { private_call }
        }

        pub fn execute_to_equal(self, expected: u32) {
            let private_call = self.private_call.to_private_circuit_public_inputs();
            // Safety: this is a test
            let index = unsafe {
                find_first_revertible_item_index(
                    private_call.min_revertible_side_effect_counter,
                    private_call.private_call_requests,
                )
            };
            assert_eq(index, expected);
        }
    }

    #[test]
    fn find_first_revertible_item_index_empty() {
        let builder = TestBuilder::new();
        builder.execute_to_equal(0);
    }

    #[test]
    fn find_first_revertible_item_index_empty_with_min_counter() {
        let mut builder = TestBuilder::new();

        builder.private_call.min_revertible_side_effect_counter = 5;

        builder.execute_to_equal(0);
    }

    #[test]
    fn find_first_revertible_item_index_only_revertible() {
        let mut builder = TestBuilder::new();

        // Revertible.
        builder.private_call.end_setup();
        builder.private_call.append_private_call_requests(3);

        builder.execute_to_equal(0);
    }

    #[test]
    fn find_first_revertible_item_index_only_non_revertible() {
        let mut builder = TestBuilder::new();

        // Non-revertible.
        builder.private_call.append_private_call_requests(2);
        builder.private_call.end_setup();

        builder.execute_to_equal(2);
    }

    #[test]
    fn find_first_revertible_item_index_both() {
        let mut builder = TestBuilder::new();

        // Non-revertible.
        builder.private_call.append_private_call_requests(2);
        // Revertible.
        builder.private_call.end_setup();
        builder.private_call.append_private_call_requests(3);

        builder.execute_to_equal(2);
    }
}
