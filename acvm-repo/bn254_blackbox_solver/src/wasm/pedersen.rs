use acir::FieldElement;

use super::{Assignments, Barretenberg, Error, FIELD_BYTES};

pub(crate) trait Pedersen {
    fn encrypt(
        &self,
        inputs: Vec<FieldElement>,
        hash_index: u32,
    ) -> Result<(FieldElement, FieldElement), Error>;

    fn hash(&self, inputs: Vec<FieldElement>, hash_index: u32) -> Result<FieldElement, Error>;
}

impl Pedersen for Barretenberg {
    fn encrypt(
        &self,
        inputs: Vec<FieldElement>,
        hash_index: u32,
    ) -> Result<(FieldElement, FieldElement), Error> {
        let input_buf = Assignments::from(inputs).to_bytes();
        let input_ptr = self.allocate(&input_buf)?;
        let result_ptr: usize = 0;

        self.call_multiple(
            "pedersen_plookup_commit_with_hash_index",
            vec![&input_ptr, &result_ptr.into(), &hash_index.into()],
        )?;

        let result_bytes: [u8; 2 * FIELD_BYTES] = self.read_memory(result_ptr);
        let (point_x_bytes, point_y_bytes) = result_bytes.split_at(FIELD_BYTES);

        let point_x = FieldElement::from_be_bytes_reduce(point_x_bytes);
        let point_y = FieldElement::from_be_bytes_reduce(point_y_bytes);

        Ok((point_x, point_y))
    }

    fn hash(&self, inputs: Vec<FieldElement>, hash_index: u32) -> Result<FieldElement, Error> {
        let input_buf = Assignments::from(inputs).to_bytes();
        let input_ptr = self.allocate(&input_buf)?;
        let result_ptr: usize = 0;

        self.call_multiple(
            "pedersen_plookup_compress_with_hash_index",
            vec![&input_ptr, &result_ptr.into(), &hash_index.into()],
        )?;

        let result_bytes: [u8; FIELD_BYTES] = self.read_memory(result_ptr);

        let hash = FieldElement::from_be_bytes_reduce(&result_bytes);

        Ok(hash)
    }
}

#[test]
fn pedersen_hash_to_point() -> Result<(), Error> {
    let barretenberg = Barretenberg::new();
    let (x, y) = barretenberg.encrypt(vec![FieldElement::one(), FieldElement::one()], 1)?;
    let expected_x = FieldElement::from_hex(
        "0x12afb43195f5c621d1d2cabb5f629707095c5307fd4185a663d4e80bb083e878",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x25793f5b5e62beb92fd18a66050293a9fd554a2ff13bceba0339cae1a038d7c1",
    )
    .unwrap();

    assert_eq!(expected_x.to_hex(), x.to_hex());
    assert_eq!(expected_y.to_hex(), y.to_hex());
    Ok(())
}
