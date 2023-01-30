fn main(message : pub str<11>, hex_as_string : str<4>, hex_as_field : Field) {
    let mut bad_message = "hello world";

    constrain message == "hello world";
    bad_message = "helld world";

    constrain message != bad_message;

    constrain hex_as_string == "0x41";
    // constrain hex_as_string != 0x41; This will fail with a type mismatch between str[4] and Field
    constrain hex_as_field == 0x41;
}