fn main(message : [char; 11]) {
    let bad_message = "helld world";

    constrain message == "hello world";

    constrain message != bad_message;
}