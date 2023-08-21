global NIBBLE_LENGTH: Field = 16;

fn compact_decode<N>(input: [u8; N], length: Field) -> ([u4; NIBBLE_LENGTH], Field)
{
    assert(2*input.len() as u64 <= NIBBLE_LENGTH as u64);
    assert(length as u64 <= input.len() as u64);
    
    let mut nibble = [0 as u4; NIBBLE_LENGTH];

    let first_nibble = (input[0] >> 4) as u4;
    let parity = first_nibble as u1;
    
    if parity == 1
    {
	    nibble[0] = (input[0] & 0x0f) as u4;
	    for i in 1..input.len()
	    {
 	        if i as u64 < length as u64
 	        {
	         	let x = input[i];
 	         	nibble[2*i - 1] = (x >> 4) as u4;
	         	nibble[2*i] = (x & 0x0f) as u4;
 	        }
	    }
    }
    else
    {
	    for i in 0..2
	    {
 	        if (i as u64) < length as u64 - 1
 	        {
		        let x = input[i + 1];
 		        nibble[2*i] = (x >> 4) as u4;
		        nibble[2*i + 1] = (x & 0x0f) as u4;
 	        }
	    }
    }

    let out = (nibble, 2*length + (parity as Field) - 2);

    out
}

fn enc<N>(value: [u8; N], value_length: Field) -> ([u8; 32], Field)
{
    assert(value.len() as u8 >= value_length as u8);
    let mut out_value = [0; 32];
    if value_length == 0
    {
    	let out = (out_value, value_length);
		out
    }
    else { if value_length as u8 < 31
    {
	out_value[0] = 0x80 + value_length as u8;
	
		for i in 1..value.len()
		{
			out_value[i] = value[i-1];
		}
		
		let out = (out_value, value_length + 1);

	out
    }
    else
    {
		let out = (out_value, 32);
		out
    }
	}
}

fn main(x: [u8; 5], z: Field)
{
    //Issue 1144
	let (nib, len) = compact_decode(x,z);
	assert_eq(len, 5);
	assert_eq([nib[0], nib[1], nib[2], nib[3], nib[4]], [15, 1, 12, 11, 8]);

	// Issue 1169
	let val1 = [0xb8,0x8f,0x61,0xe6,0xfb,0xda,0x83,0xfb,0xff,0xfa,0xbe,0x36,0x41,0x12,0x13,0x74,0x80,0x39,0x80,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00];
    let val1_length = 20;

    let enc_val1 = enc(val1,val1_length);

    assert_eq(enc_val1.0, [0x94,0xb8,0x8f,0x61,0xe6,0xfb,0xda,0x83,0xfb,0xff,0xfa,0xbe,0x36,0x41,0x12,0x13,0x74,0x80,0x39,0x80,0x18,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00]);
    assert_eq(enc_val1.1, 21);

}
