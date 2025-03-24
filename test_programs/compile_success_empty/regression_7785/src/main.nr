mod from_all;
mod common;
mod email_addr;

global ADDR_REGEX_MAX_CAPTURE_LEN: u32 = 320;
global CRLF_POSTFIX: u32 = 2;

global FROM_ALL_MAX_PREFIX: u32 = 7;
global FROM_ALL_MAX_LEN: u32 = FROM_ALL_MAX_PREFIX + ADDR_REGEX_MAX_CAPTURE_LEN + CRLF_POSTFIX;

fn main() {
    let input = "\r\nto:from:Sora Suegami <suegamisora@gmail.com>\r\n".as_bytes();
    let _ = regex_match(input);
}

fn regex_match<let N: u32>(input: [u8; N]) {
    let (from_all_sequence, from_all_match) = from_all::regex_match(input);
    assert(from_all_sequence.len() == 1, "Expected sequence found to from_all match");
    assert(from_all_match, "Expected match found to from_all match");
    let from_all_substring: BoundedVec<u8, FROM_ALL_MAX_LEN> =
        common::extract_substring(from_all_sequence.get_unchecked(0), input);

    let _ = email_addr::regex_match(from_all_substring.storage());
}
