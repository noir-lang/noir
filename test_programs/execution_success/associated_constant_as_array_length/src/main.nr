// An associated constant accessed by its qualified name (`Packet::SIZE`) must carry its
// declared numeric type, so it can be used in a global and then as an array length in
// `main`'s signature. Otherwise the global's type falls through to `Type::Error` and the
// array length is rejected as an invalid entry-point type.
trait HasSize {
    let SIZE: u32;
}

struct Packet {}

impl HasSize for Packet {
    let SIZE: u32 = 4;
}

global N: u32 = Packet::SIZE;

fn main(_a: [Field; N]) {}
