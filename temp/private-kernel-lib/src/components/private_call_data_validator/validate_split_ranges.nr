use dep::types::abis::side_effect::RangeOrdered;

pub fn validate_split_ranges<T, let N: u32>(
    min_revertible_side_effect_counter: u32,
    first_revertible_item_index: u32,
    items: [T; N],
    num_items: u32,
)
where
    T: RangeOrdered,
{
    if first_revertible_item_index != 0 {
        let last_non_revertible_item_index = first_revertible_item_index - 1;
        let item = items[last_non_revertible_item_index];
        assert(
            min_revertible_side_effect_counter > item.counter_end(),
            "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item",
        );
    }
    if first_revertible_item_index != num_items {
        let item = items[first_revertible_item_index];
        assert(
            min_revertible_side_effect_counter <= item.counter_start(),
            "min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible item",
        );
    }
}

mod tests {
    use crate::components::private_call_data_validator::validate_split_ranges::validate_split_ranges;
    use dep::types::tests::fixture_builder::FixtureBuilder;

    struct TestBuilder {
        private_call: FixtureBuilder,
        first_revertible_private_call_request_index: u32,
    }

    impl TestBuilder {
        pub fn new() -> Self {
            let private_call = FixtureBuilder::new();
            TestBuilder { private_call, first_revertible_private_call_request_index: 0 }
        }

        pub fn split_calls(&mut self, counter: u32) {
            self.private_call.min_revertible_side_effect_counter = counter;
            self.first_revertible_private_call_request_index =
                self.private_call.private_call_requests.len();
        }

        pub fn add_private_call_request(&mut self, counter_start: u32, counter_end: u32) {
            let index = self.private_call.private_call_requests.len();
            self.private_call.append_private_call_requests(1);
            let mut call_request = self.private_call.private_call_requests.get(index);
            call_request.start_side_effect_counter = counter_start;
            call_request.end_side_effect_counter = counter_end;
            self.private_call.private_call_requests.set(index, call_request);
            self.private_call.counter = counter_end + 1;
        }

        pub fn execute(self) {
            validate_split_ranges(
                self.private_call.min_revertible_side_effect_counter,
                self.first_revertible_private_call_request_index,
                self.private_call.private_call_requests.storage(),
                self.private_call.private_call_requests.len(),
            );
        }
    }

    #[test]
    fn validate_split_ranges_succeeds() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        builder.split_calls(60);
        builder.add_private_call_request(60, 70);

        builder.execute();
    }

    #[test]
    fn validate_split_ranges_empty_revertible_succeeds() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        builder.split_calls(51);

        builder.execute();
    }

    #[test]
    fn validate_split_ranges_empty_non_revertible_succeeds() {
        let mut builder = TestBuilder::new();

        builder.split_calls(20);
        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);

        builder.execute();
    }

    #[test]
    fn validate_split_ranges_less_than_first_revertible_success() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        // Tweak the counter to be less than the start counter of the first revertible call.
        builder.split_calls(59);
        builder.add_private_call_request(60, 70);

        builder.execute();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
    fn validate_split_ranges_less_than_last_non_revertible_fails() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        // Tweak the counter to be less than the end counter of the last non-revertible call.
        builder.split_calls(49);
        builder.add_private_call_request(60, 70);

        builder.execute();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
    fn validate_split_ranges_equal_last_non_revertible_fails() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        // Tweak the counter to equal the end counter of the last non-revertible call.
        builder.split_calls(50);

        builder.execute();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible item")]
    fn validate_split_ranges_greater_than_first_revertible_fails() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        // Tweak the counter to be greater than the start counter of the first revertible call.
        builder.split_calls(61);
        builder.add_private_call_request(60, 70);

        builder.execute();
    }

    #[test]
    fn validate_split_ranges_0_succeeds() {
        let mut builder = TestBuilder::new();

        // Set the counter to be 0.
        builder.split_calls(0);
        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);

        builder.execute();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible item")]
    fn validate_split_ranges_0_wrong_hint_fails() {
        let mut builder = TestBuilder::new();

        builder.split_calls(0);
        // Set the index hint to be 1.
        builder.first_revertible_private_call_request_index = 1;
        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);

        builder.execute();
    }

    #[test(should_fail_with = "min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible item")]
    fn validate_split_ranges_index_hint_greater_than_len_fails() {
        let mut builder = TestBuilder::new();

        builder.add_private_call_request(20, 30);
        builder.add_private_call_request(40, 50);
        builder.split_calls(51);
        // Increase the index by 1.
        builder.first_revertible_private_call_request_index += 1;

        builder.execute();
    }
}
