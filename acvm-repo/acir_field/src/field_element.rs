    proptest! {
        #[test]
        fn test_endianness_prop(value in any::<u64>()) {
            let field = FieldElement::<ark_bn254::Fr>::from(value);
            // Test serialization consistency
            let le_bytes = field.to_le_bytes();
            let be_bytes = field.to_be_bytes();

            let mut reversed_le = le_bytes.clone();
            reversed_le.reverse();
            prop_assert_eq!(&be_bytes, &reversed_le, "BE bytes should be reverse of LE bytes");

            // Test deserialization consistency
            let from_le = FieldElement::from_le_bytes_reduce(&le_bytes);
            let from_be = FieldElement::from_be_bytes_reduce(&be_bytes);
            prop_assert_eq!(from_le, from_be, "Deserialization should be consistent between LE and BE");
            prop_assert_eq!(from_le, field, "Deserialized value should match original");
        }
    }

    #[test]
    fn test_endianness() {
        let field = FieldElement::<ark_bn254::Fr>::from(0x1234_5678_u32);
        let le_bytes = field.to_le_bytes();
        let be_bytes = field.to_be_bytes();

        // Check that the bytes are reversed between BE and LE
        let mut reversed_le = le_bytes.clone();
        reversed_le.reverse();
        assert_eq!(&be_bytes, &reversed_le);

        // Verify we can reconstruct the same field element from either byte order
        let from_le = FieldElement::from_le_bytes_reduce(&le_bytes);
        let from_be = FieldElement::from_be_bytes_reduce(&be_bytes);
        assert_eq!(from_le, from_be);
        assert_eq!(from_le, field);

        // Additional test with a larger number to ensure proper byte handling
        let large_field = FieldElement::<ark_bn254::Fr>::from(0x0123_4567_89AB_CDEF_u64);
        let large_le = large_field.to_le_bytes();
        let reconstructed = FieldElement::from_le_bytes_reduce(&large_le);
        assert_eq!(reconstructed, large_field);
    }

    proptest! {
