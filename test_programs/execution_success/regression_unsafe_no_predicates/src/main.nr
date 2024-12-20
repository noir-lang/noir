fn main(x: u8, nest: bool) {
    if nest {
        let foo = unsafe_assert([x]);
        assert(foo != 0);
    }
}

#[no_predicates]
pub fn unsafe_assert<let N: u32>(msg: [u8; N]) -> u8 {
    let block = unsafe {
        //@safety: testing context
        get_block(msg)
    };
    verify_block(msg, block);
    block[0]
}

unconstrained fn get_block<let N: u32>(msg: [u8; N]) -> [u8; 2] {
    let mut block: [u8; 2] = [0; 2];
    block[0] = msg[0];
    block
}

fn verify_block<let N: u32>(msg: [u8; N], block: [u8; 2]) {
    assert_eq(block[0], msg[0]);
}
