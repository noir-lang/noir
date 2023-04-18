global NIBBLE_LENGTH: comptime Field = 16;

fn compact_decode<N>(input: [u8; N], length: Field) -> ([u4; NIBBLE_LENGTH], Field)
{
    constrain 2*input.len() as u64 <= NIBBLE_LENGTH as u64;
    constrain length as u64 <= input.len() as u64;
    
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

fn main(x: [u8; 5], z: Field)
{
    //Issue 1144
	let (nib, len) = compact_decode(x,z);
	constrain len == 5;
	constrain [nib[0], nib[1], nib[2], nib[3], nib[4]] == [15, 1, 12, 11, 8];
}

#[test]
fn test_1144()
{
    main([0x3f, 0x1c, 0xb8, 0x99, 0xab], 3);
}