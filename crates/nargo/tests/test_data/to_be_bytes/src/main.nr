use dep::std;

fn main(x : Field) {
    // The result of this byte array will be big-endian
    let be_byte_array = x.to_be_bytes(31);
    // The result of this byte array will be little-endian
    let le_byte_array = x.to_le_bytes(31);
    
    constrain be_byte_array[253] == le_byte_array[0];
    constrain be_byte_array[252] == le_byte_array[1];
    constrain be_byte_array[251] == le_byte_array[2];
}

