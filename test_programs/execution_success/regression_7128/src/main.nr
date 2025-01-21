fn main(in0: Field) -> pub Field {
    let mut out0 : Field = 0;
    let mut tmp1 : Field = 0;

    if ((out0 == out0) & true) // <== changing out0 to in0 or removing
    {                          //     the comparison changes the result
        let in0_as_bytes : [u8; 32] = in0.to_be_bytes();
        let mut result : [u8; 32] = 0.to_be_bytes();
        for i in 0..32 {
            result[i] = (in0_as_bytes[i] ^ result[i]);
        }
        tmp1 = std::field::bytes32_to_field(result);
    }

    let mut tmp2 : Field = 0; // <== moving this to the top of main,
    if (0.lt(in0))            //     changes the result
    {
        tmp2 = 1;
    }

    out0 = (tmp2 - tmp1);

    assert(out0 == 0, "completeness violation");

    out0
}