use super::{Barretenberg, Error};

use acvm::FieldElement;

pub(crate) trait Pedersen {
    fn encrypt(
        &self,
        inputs: Vec<FieldElement>,
        hash_index: u32,
    ) -> Result<(FieldElement, FieldElement), Error>;
}

#[cfg(feature = "native")]
impl Pedersen for Barretenberg {
    fn encrypt(
        &self,
        inputs: Vec<FieldElement>,
        hash_index: u32,
    ) -> Result<(FieldElement, FieldElement), Error> {
        use super::native::field_to_array;

        let mut inputs_buf = Vec::new();
        for f in inputs {
            inputs_buf.push(field_to_array(&f)?);
        }
        let (point_x_bytes, point_y_bytes) =
            barretenberg_sys::pedersen::encrypt(&inputs_buf, hash_index);

        let point_x = FieldElement::from_be_bytes_reduce(&point_x_bytes);
        let point_y = FieldElement::from_be_bytes_reduce(&point_y_bytes);

        Ok((point_x, point_y))
    }
}

#[cfg(not(feature = "native"))]
impl Pedersen for Barretenberg {
    fn encrypt(
        &self,
        inputs: Vec<FieldElement>,
        hash_index: u32,
    ) -> Result<(FieldElement, FieldElement), Error> {
        use super::barretenberg_structures::Assignments;
        use super::FIELD_BYTES;

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
}

#[test]
fn pedersen_hash_to_point() -> Result<(), Error> {
    let barretenberg = Barretenberg::new();
    let (x, y) = barretenberg.encrypt(vec![FieldElement::zero(), FieldElement::one()], 0)?;
    let expected_x = FieldElement::from_hex(
        "0x0c5e1ddecd49de44ed5e5798d3f6fb7c71fe3d37f5bee8664cf88a445b5ba0af",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x230294a041e26fe80b827c2ef5cb8784642bbaa83842da2714d62b1f3c4f9752",
    )
    .unwrap();

    assert_eq!(expected_x.to_hex(), x.to_hex());
    assert_eq!(expected_y.to_hex(), y.to_hex());
    Ok(())
}
