use crate::acir_field::AcirField;
use num_bigint::BigUint;
use num_traits::Num;

pub(crate) fn parse_str_to_field<F: AcirField>(value: &str) -> Result<F, String> {
    // get the sign
    let is_negative = value.trim().starts_with("-");
    let unsigned_value_string =
        if is_negative { value.strip_prefix("-").unwrap().trim() } else { value.trim() };

    let big_num = if let Some(hex) = unsigned_value_string.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(unsigned_value_string, 10)
    };

    big_num.map_err(|_| "could not convert string to field".to_string()).map(|num| {
        if is_negative {
            -F::from_be_bytes_reduce(&num.to_bytes_be())
        } else {
            F::from_be_bytes_reduce(&num.to_bytes_be())
        }
    })
}

pub(crate) fn clean_string(string: &str) -> String {
    let mut result_string = string.replace(" ", "").replace("\n", "");
    if string.starts_with("[") && string.ends_with("]") {
        result_string =
            result_string.strip_prefix("[").unwrap().strip_suffix("]").unwrap().to_string();
    }
    result_string
}
