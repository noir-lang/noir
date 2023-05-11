use dep::std;

fn main(message : [u8;38],hashed_message : [u8;32]) {
    assert(hashed_message == std::hash::sha256(message));
}