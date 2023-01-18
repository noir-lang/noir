use dep::std;

fn main(message : pub str[11], y : Field) {
    let mut bad_message = "hello world";

    constrain message == "hello world";
    bad_message = "helld world";
    let x = 10;
    let z = x * 5;
    std::println(10);

    std::println(z); // x * 5 in println not yet supported
    std::println(x);

    let array = [1, 2, 3, 5, 8];
    constrain y == 5; // Change to y != 5 to see how the later print statements are not called
    std::println(array);

    std::println(bad_message);
    constrain message != bad_message;

    let hash = std::hash::pedersen([x]);
    std::println(hash);
}