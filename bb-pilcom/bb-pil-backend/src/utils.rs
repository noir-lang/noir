use num_bigint::BigInt;
use powdr_number::FieldElement;

/// Sanitize Names
///
/// Column titles that we get from pil contain . to distinguish which pil namespace they belong to
/// We need to replace these with _ to make them valid C++ identifiers
pub fn sanitize_name(string: &str) -> String {
    string.replace(['.', '[', ']'], "_")
}

/// Flatten
///
/// Returns a flattened concatenation of the input arrays
pub fn flatten(list: &[Vec<String>]) -> Vec<String> {
    let arr = list.iter().cloned();
    arr.into_iter().flatten().collect()
}

/// Snake Case
///
/// Transform camel case string into snake case, such as: RedFlower --> red_flower
pub fn snake_case(input: &str) -> String {
    let mut result = String::new();

    // Handle the first character
    if input.is_empty() {
        return result; // Empty input
    }
    let mut first_char = input.chars().next().unwrap();
    if first_char.is_uppercase() {
        first_char = first_char.to_ascii_lowercase();
    }
    result.push(first_char);

    // Process remaining characters
    for ch in input.chars().skip(1) {
        if ch.is_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn sort_cols(cols: &[String]) -> Vec<String> {
    let mut cols = cols.to_vec();
    cols.sort();
    cols
}

pub fn format_field<F: FieldElement>(n: &F) -> String {
    let number: BigInt = BigInt::from_bytes_le(num_bigint::Sign::Plus, &n.to_bytes_le());
    if number.bits() < 32 {
        format!("FF({})", number)
    } else if number.bits() < 64 {
        format!("FF({}UL)", number)
    } else {
        // It's ok to use a string here since the constructor is constexpr.
        // I.e., things will get resolved efficiently at compile-time.
        // format!("FF(\"{:0>64}\")", number.to_str_radix(16))
        let bytes = n.to_arbitrary_integer().to_be_bytes();
        let padding_len = 32 - bytes.len();

        let mut padded_bytes = vec![0; padding_len];
        padded_bytes.extend_from_slice(&bytes);

        let mut chunks: Vec<u64> = padded_bytes
            .chunks(8)
            .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
            .collect();

        chunks.resize(4, 0);
        format!(
            "FF(uint256_t{{{}UL, {}UL, {}UL, {}UL}})",
            chunks[3], chunks[2], chunks[1], chunks[0]
        )
    }
}
