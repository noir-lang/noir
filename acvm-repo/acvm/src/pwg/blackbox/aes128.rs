use acir::{
    AcirField,
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::aes128_encrypt;

use crate::{OpcodeResolutionError, pwg::insert_value};

use super::utils::{to_u8_array, to_u8_vec};

pub(super) fn solve_aes128_encryption_opcode<F: AcirField>(
    initial_witness: &mut WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    iv: &[FunctionInput<F>; 16],
    key: &[FunctionInput<F>; 16],
    outputs: &[Witness],
) -> Result<(), OpcodeResolutionError<F>> {
    let ciphertext = execute_aes128_encryption_opcode(initial_witness, inputs, iv, key)?;

    assert_eq!(
        outputs.len(),
        ciphertext.len(),
        "Number of outputs does not match number of ciphertext bytes"
    );

    // Write witness assignments
    for (output_witness, value) in outputs.iter().zip(ciphertext.into_iter()) {
        insert_value(output_witness, F::from(u128::from(value)), initial_witness)?;
    }

    Ok(())
}

pub(crate) fn execute_aes128_encryption_opcode<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    inputs: &[FunctionInput<F>],
    iv: &[FunctionInput<F>; 16],
    key: &[FunctionInput<F>; 16],
) -> Result<Vec<u8>, OpcodeResolutionError<F>> {
    let scalars = to_u8_vec(initial_witness, inputs)?;

    let iv = to_u8_array(initial_witness, iv)?;
    let key = to_u8_array(initial_witness, key)?;

    let ciphertext = aes128_encrypt(&scalars, iv, key)?;

    Ok(ciphertext)
}

#[cfg(test)]
mod tests {
    use crate::pwg::blackbox::solve_aes128_encryption_opcode;
    use acir::{
        FieldElement,
        circuit::opcodes::FunctionInput,
        native_types::{Witness, WitnessMap},
    };
    use std::collections::BTreeMap;

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_aes() {
        // Test vector is coming from Barretenberg (cf. aes128.test.cpp)
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            // Key { 0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c }
            (Witness(1), FieldElement::from(0x2bu128)),
            (Witness(2), FieldElement::from(0x7eu128)),
            (Witness(3), FieldElement::from(0x15u128)),
            (Witness(4), FieldElement::from(0x16u128)),
            (Witness(5), FieldElement::from(0x28u128)),
            (Witness(6), FieldElement::from(0xaeu128)),
            (Witness(7), FieldElement::from(0xd2u128)),
            (Witness(8), FieldElement::from(0xa6u128)),
            (Witness(9), FieldElement::from(0xabu128)),
            (Witness(10), FieldElement::from(0xf7u128)),
            (Witness(11), FieldElement::from(0x15u128)),
            (Witness(12), FieldElement::from(0x88u128)),
            (Witness(13), FieldElement::from(0x09u128)),
            (Witness(14), FieldElement::from(0xcfu128)),
            (Witness(15), FieldElement::from(0x4fu128)),
            (Witness(16), FieldElement::from(0x3cu128)),
            // IV {0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f }
            (Witness(17), FieldElement::from(0x00u128)),
            (Witness(18), FieldElement::from(0x01u128)),
            (Witness(19), FieldElement::from(0x02u128)),
            (Witness(20), FieldElement::from(0x03u128)),
            (Witness(21), FieldElement::from(0x04u128)),
            (Witness(22), FieldElement::from(0x05u128)),
            (Witness(23), FieldElement::from(0x06u128)),
            (Witness(24), FieldElement::from(0x07u128)),
            (Witness(25), FieldElement::from(0x08u128)),
            (Witness(26), FieldElement::from(0x09u128)),
            (Witness(27), FieldElement::from(0x0au128)),
            (Witness(28), FieldElement::from(0x0bu128)),
            (Witness(29), FieldElement::from(0x0cu128)),
            (Witness(30), FieldElement::from(0x0du128)),
            (Witness(31), FieldElement::from(0x0eu128)),
            (Witness(32), FieldElement::from(0x0fu128)),
            // Input { 0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
            //        0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51,
            //        0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef,
            //        0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10 };
            (Witness(33), FieldElement::from(0x6bu128)),
            (Witness(34), FieldElement::from(0xc1u128)),
            (Witness(35), FieldElement::from(0xbeu128)),
            (Witness(36), FieldElement::from(0xe2u128)),
            (Witness(37), FieldElement::from(0x2eu128)),
            (Witness(38), FieldElement::from(0x40u128)),
            (Witness(39), FieldElement::from(0x9fu128)),
            (Witness(40), FieldElement::from(0x96u128)),
            (Witness(41), FieldElement::from(0xe9u128)),
            (Witness(42), FieldElement::from(0x3du128)),
            (Witness(43), FieldElement::from(0x7eu128)),
            (Witness(44), FieldElement::from(0x11u128)),
            (Witness(45), FieldElement::from(0x73u128)),
            (Witness(46), FieldElement::from(0x93u128)),
            (Witness(47), FieldElement::from(0x17u128)),
            (Witness(48), FieldElement::from(0x2au128)),
            (Witness(49), FieldElement::from(0xaeu128)),
            (Witness(50), FieldElement::from(0x2du128)),
            (Witness(51), FieldElement::from(0x8au128)),
            (Witness(52), FieldElement::from(0x57u128)),
            (Witness(53), FieldElement::from(0x1eu128)),
            (Witness(54), FieldElement::from(0x03u128)),
            (Witness(55), FieldElement::from(0xacu128)),
            (Witness(56), FieldElement::from(0x9cu128)),
            (Witness(57), FieldElement::from(0x9eu128)),
            (Witness(58), FieldElement::from(0xb7u128)),
            (Witness(59), FieldElement::from(0x6fu128)),
            (Witness(60), FieldElement::from(0xacu128)),
            (Witness(61), FieldElement::from(0x45u128)),
            (Witness(62), FieldElement::from(0xafu128)),
            (Witness(63), FieldElement::from(0x8eu128)),
            (Witness(64), FieldElement::from(0x51u128)),
            (Witness(65), FieldElement::from(0x30u128)),
            (Witness(66), FieldElement::from(0xc8u128)),
            (Witness(67), FieldElement::from(0x1cu128)),
            (Witness(68), FieldElement::from(0x46u128)),
            (Witness(69), FieldElement::from(0xa3u128)),
            (Witness(70), FieldElement::from(0x5cu128)),
            (Witness(71), FieldElement::from(0xe4u128)),
            (Witness(72), FieldElement::from(0x11u128)),
            (Witness(73), FieldElement::from(0xe5u128)),
            (Witness(74), FieldElement::from(0xfbu128)),
            (Witness(75), FieldElement::from(0xc1u128)),
            (Witness(76), FieldElement::from(0x19u128)),
            (Witness(77), FieldElement::from(0x1au128)),
            (Witness(78), FieldElement::from(0x0au128)),
            (Witness(79), FieldElement::from(0x52u128)),
            (Witness(80), FieldElement::from(0xefu128)),
            (Witness(81), FieldElement::from(0xf6u128)),
            (Witness(82), FieldElement::from(0x9fu128)),
            (Witness(83), FieldElement::from(0x24u128)),
            (Witness(84), FieldElement::from(0x45u128)),
            (Witness(85), FieldElement::from(0xdfu128)),
            (Witness(86), FieldElement::from(0x4fu128)),
            (Witness(87), FieldElement::from(0x9bu128)),
            (Witness(88), FieldElement::from(0x17u128)),
            (Witness(89), FieldElement::from(0xadu128)),
            (Witness(90), FieldElement::from(0x2bu128)),
            (Witness(91), FieldElement::from(0x41u128)),
            (Witness(92), FieldElement::from(0x7bu128)),
            (Witness(93), FieldElement::from(0xe6u128)),
            (Witness(94), FieldElement::from(0x6cu128)),
            (Witness(95), FieldElement::from(0x37u128)),
            (Witness(96), FieldElement::from(0x10u128)),
        ]));
        let mut inputs = [FunctionInput::Witness(Witness(0)); 64];
        for i in 0..64 {
            inputs[i] = FunctionInput::Witness(Witness(33 + i as u32));
        }
        let mut iv = [FunctionInput::Witness(Witness(0)); 16];
        for i in 0..16 {
            iv[i] = FunctionInput::Witness(Witness(17 + i as u32));
        }
        let mut key = [FunctionInput::Witness(Witness(0)); 16];
        for i in 0..16 {
            key[i] = FunctionInput::Witness(Witness(1 + i as u32));
        }
        let mut outputs = vec![];
        for i in 97..161 {
            outputs.push(Witness(i));
        }

        solve_aes128_encryption_opcode(&mut initial_witness, &inputs, &iv, &key, &outputs).unwrap();
        let expected_output: [u128; 64] = [
            0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9,
            0x19, 0x7d, 0x50, 0x86, 0xcb, 0x9b, 0x50, 0x72, 0x19, 0xee, 0x95, 0xdb, 0x11, 0x3a,
            0x91, 0x76, 0x78, 0xb2, 0x73, 0xbe, 0xd6, 0xb8, 0xe3, 0xc1, 0x74, 0x3b, 0x71, 0x16,
            0xe6, 0x9e, 0x22, 0x22, 0x95, 0x16, 0x3f, 0xf1, 0xca, 0xa1, 0x68, 0x1f, 0xac, 0x09,
            0x12, 0x0e, 0xca, 0x30, 0x75, 0x86, 0xe1, 0xa7,
        ];
        let expected_output = expected_output.map(FieldElement::from);
        let expected_output: Vec<&FieldElement> = expected_output.iter().collect();
        for i in 0..64 {
            assert_eq!(initial_witness[&Witness(97 + i as u32)], *expected_output[i]);
        }
    }
}
