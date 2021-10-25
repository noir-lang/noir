use noir_field::FieldElement;

use super::Barretenberg;
use super::BARRETENBERG;
use super::field_to_array;

impl Barretenberg {
    pub fn fixed_base(&mut self, input: &FieldElement) -> (FieldElement, FieldElement) {
        let _m = BARRETENBERG.lock().unwrap();
        let result_bytes = barretenberg_wrapper::schnorr::construct_public_key(
            &field_to_array(input),
        );
        let (pubkey_x_bytes, pubkey_y_bytes) = result_bytes.split_at(32);
        let pubkey_x = FieldElement::from_be_bytes_reduce(pubkey_x_bytes);
        let pubkey_y = FieldElement::from_be_bytes_reduce(pubkey_y_bytes);
        (pubkey_x, pubkey_y)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn smoke_test() {
        let mut barretenberg = Barretenberg::new();
        let input = FieldElement::one();

        let res = barretenberg.fixed_base(&input);
        let x = "0000000000000000000000000000000000000000000000000000000000000001";
        let y = "0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
    }
}
