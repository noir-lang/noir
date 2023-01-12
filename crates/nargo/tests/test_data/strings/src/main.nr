use dep::std;

fn main(message : pub str[11], y : Field) {
    let mut bad_message = "hello world";

    constrain message == "hello world";
    bad_message = "helld world";
    let x = 2;
    log(x);

    // Currently broken
    // let hash = std::hash::pedersen([x])[0];
    // log(bad_message);
    log("five");

    let array = [1, 2, 3, 5, 8];
    constrain y != 5;
    log(array);

    constrain message != bad_message;
}