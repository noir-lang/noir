use dep::std;

fn main(x : Field)  {
    // The result of this byte array will be big-endian
    let y: Field = 2040124;
    let be_byte_array = y.to_le_bytes(31);
    // The result of this byte array will be little-endian
    let le_byte_array = x.to_be_bytes(31);
    
    constrain be_byte_array[0] == le_byte_array[30];
    constrain be_byte_array[1] == le_byte_array[29];
    constrain be_byte_array[2] == le_byte_array[28];
}