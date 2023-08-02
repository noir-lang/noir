use dep::std;

fn main(message : pub str<11>, y : Field, hex_as_string : str<4>, hex_as_field : Field) {
    let mut bad_message = "hello world";

    assert(message == "hello world");
    bad_message = "helld world";
    let x = 10;
    let z = x * 5;
    std::println(10);

    std::println(z); // x * 5 in println not yet supported
    std::println(x);

    let array = [1, 2, 3, 5, 8];
    assert(y == 5); // Change to y != 5 to see how the later print statements are not called
    std::println(array);

    std::println(bad_message);
    assert(message != bad_message);

    let hash = std::hash::pedersen([x]);
    std::println(hash);
    
    assert(hex_as_string == "0x41");
    // assert(hex_as_string != 0x41); This will fail with a type mismatch between str[4] and Field
    assert(hex_as_field == 0x41);
}

#[test]
fn test_prints_strings() {
    let message = "hello world!";

    std::println(message);
    std::println("goodbye world");
}

#[test]
fn test_prints_array() {
    let array = [1, 2, 3, 5, 8];

    let s = Test { a: 1, b: 2, c: [3, 4] };
    std::println(s);

    std::println(array);

    let hash = std::hash::pedersen(array);
    std::println(hash);
}

fn failed_constraint(hex_as_field: Field) {
    // TODO(#2116): Note that `println` will not work if a failed constraint can be 
    // evaluated at compile time. 
    // When this method is called from a test method or with constant values
    // a `Failed constraint` compile error will be caught before this `println`
    // is executed as the input will be a constant. 
    std::println(hex_as_field);
    assert(hex_as_field != 0x41);
}

#[test]
fn test_failed_constraint() {
    failed_constraint(0x41);
}

struct Test {
    a: Field,
    b: Field,
    c: [Field; 2],
}
