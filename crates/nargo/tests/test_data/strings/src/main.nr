fn main(message : pub str[11]) {
    let mut bad_message = "hello world";

    constrain message == "hello world";
    bad_message = "helld world";

    constrain message != bad_message;
}