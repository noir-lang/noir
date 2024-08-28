fn main(
    msg_just_over_block: [u8; 68],
    result_just_over_block: pub [u8; 32],
    msg_multiple_of_block: [u8; 448],
    result_multiple_of_block: pub [u8; 32],
    // We want to make sure we are testing a message with a size >= 57 but < 64
    msg_just_under_block: [u8; 60],
    result_just_under_block: pub [u8; 32],
    msg_big_not_block_multiple: [u8; 472],
    result_big: pub [u8; 32]
) {
    let hash = std::hash::sha256_var(msg_just_over_block, msg_just_over_block.len() as u64);
    assert_eq(hash, result_just_over_block);

    let hash = std::hash::sha256_var(msg_multiple_of_block, msg_multiple_of_block.len() as u64);
    assert_eq(hash, result_multiple_of_block);

    let hash = std::hash::sha256_var(msg_just_under_block, msg_just_under_block.len() as u64);
    assert_eq(hash, result_just_under_block);

    let hash = std::hash::sha256_var(
        msg_big_not_block_multiple,
        msg_big_not_block_multiple.len() as u64
    );
    assert_eq(hash, result_big);
}
