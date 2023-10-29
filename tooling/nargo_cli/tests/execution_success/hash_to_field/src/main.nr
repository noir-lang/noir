use dep::std;

fn main(input : Field) -> pub Field {
    let expected = std::hash::hash_to_field([input]);

    let input_bytes = input.to_le_bytes(32);
    let blake2s = std::hash::blake2s(input_bytes);
    let got = dep::std::field::bytes32_to_field(blake2s);

    assert(expected == got);
    expected
}