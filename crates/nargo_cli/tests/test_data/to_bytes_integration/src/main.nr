use dep::std;

fn main(x : Field)  {
    // The result of this byte array will be big-endian
    let y: Field = 2040124;
    let be_byte_array = y.to_be_bytes(31);
    // The result of this byte array will be little-endian
    let le_byte_array = x.to_le_bytes(31);
    
    constrain le_byte_array[0] == 60;
    constrain le_byte_array[0] == be_byte_array[30];
    constrain le_byte_array[1] == be_byte_array[29];
    constrain le_byte_array[2] == be_byte_array[28];
}