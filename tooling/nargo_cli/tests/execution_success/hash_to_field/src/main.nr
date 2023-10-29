use dep::std;

fn main(input : Field) -> pub Field {
    let expected = std::hash::hash_to_field([input]);
    let got = std::hash::hash_to_field_native([input]);
    assert(expected == got);
    expected
}