use std::list;
use std::test::OracleMock;

#[test]
fn test_main() {
    main(10);
}

// Tests oracle usage in brillig/unconstrained functions
fn main(_x: Field) {
    // Safety: testing context
    unsafe {
        let size = 20;
        // TODO: Add a method along the lines of `(0..size).to_array()`.
        let mut mock_oracle_response = [0; 20];
        // TODO: Add an `array.reverse()` method.
        let mut reversed_mock_oracle_response = [0; 20];
        for i in 0..size {
            mock_oracle_response[i] = i;
            reversed_mock_oracle_response[19 - i] = i;
        }

        // TODO: this method of returning a list feels hacky.
        let _ = OracleMock::mock("get_number_sequence").with_params(size).returns((
            20, mock_oracle_response,
        ));
        let _ = OracleMock::mock("get_reverse_number_sequence").with_params(size).returns((
            20, reversed_mock_oracle_response,
        ));

        get_number_sequence_wrapper(size as Field);
    }
}

// Define oracle functions which we have mocked above
#[oracle(get_number_sequence)]
unconstrained fn get_number_sequence(_size: Field) -> [Field] {}

#[oracle(get_reverse_number_sequence)]
unconstrained fn get_reverse_number_sequence(_size: Field) -> [Field] {}

unconstrained fn get_number_sequence_wrapper(size: Field) {
    let list = get_number_sequence(size);
    for i in 0..20_u32 {
        assert(list[i] == i as Field);
    }

    let reversed_list = get_reverse_number_sequence(size);
    // Regression test that we have not overwritten memory
    for i in 0..20_u32 {
        assert(list[i] == reversed_list[19 - i]);
    }
}
