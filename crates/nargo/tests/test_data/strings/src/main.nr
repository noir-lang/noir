use dep::std;

fn main(message : pub str[11], y : Field) {
    let mut bad_message = "hello world";

    constrain message == "hello world";
    bad_message = "helld world";
    let x = 10;
    let z = x * 5;
    std::println(x);
    std::println(z);
    std::println(bad_message);

    // Currently broken
    // let hash = std::hash::pedersen([x])[0];
    // std::println("five");

    let array = [1, 2, 3, 5, 8];
    constrain y == 5;
    std::println(array);

    constrain message != bad_message;
}