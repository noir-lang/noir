use dep::std;

fn main(message : [10]u8, pub_key_x : Field, pub_key_y : Field, signature : [64]u8) {
    let mut x = std::schnorr::verify_signature(pub_key_x,pub_key_y,signature, message);
    x = 3; 

    let my_const = 2;
    let foo: const u32 = my_const as u32 + 1;

    // message = [0, 1, 2, 3, ...]
    constrain message[foo as Field] == x;
}
