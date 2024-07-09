contract TestLog {
    use dep::aztec::prelude::PrivateSet;
    use dep::aztec::protocol_types::{traits::Serialize, point::Point, scalar::Scalar, address::AztecAddress};
    use dep::value_note::value_note::ValueNote;
    use dep::aztec::encrypted_logs::incoming_body::EncryptedLogIncomingBody;
    use dep::aztec::event::event_interface::EventInterface;
    use dep::aztec::encrypted_logs::encrypted_event_emission::encode_and_encrypt_event_with_randomness;
    use dep::aztec::unencrypted_logs::unencrypted_event_emission::encode_event;

    #[aztec(event)]
    struct ExampleEvent0 {
        value0: Field,
        value1: Field,
    }

    #[aztec(event)]
    struct ExampleEvent1 {
        value2: AztecAddress,
        value3: u8,
    }

    #[aztec(storage)]
    struct Storage {
        example_set: PrivateSet<ValueNote>,
    }

    // EXAMPLE_EVENT_0_BYTES_LEN + 16
    global EXAMPLE_EVENT_0_CIPHERTEXT_BYTES_LEN = 144;

    #[aztec(private)]
    fn compute_incoming_log_body_ciphertext(
        secret: Scalar,
        point: Point,
        randomness: Field,
        event_type_id: Field,
        preimage: [Field; 2]
    ) -> [u8; EXAMPLE_EVENT_0_CIPHERTEXT_BYTES_LEN] {
        EncryptedLogIncomingBody::from_event(
            ExampleEvent0 { value0: preimage[0], value1: preimage[1] },
            randomness
        ).compute_ciphertext(secret, point).as_array()
    }

    #[aztec(private)]
    fn emit_encrypted_events(other: AztecAddress, randomness: [Field; 2], preimages: [Field; 4]) {
        let event0 = ExampleEvent0 { value0: preimages[0], value1: preimages[1] };

        event0.emit(
            encode_and_encrypt_event_with_randomness(
                &mut context,
                randomness[0],
                // outgoing is set to other, incoming is set to msg sender
                other,
                context.msg_sender()
            )
        );

        // We duplicate the emission, but specifying different incoming and outgoing parties
        event0.emit(
            encode_and_encrypt_event_with_randomness(
                &mut context,
                randomness[0],
                // outgoing is set to msg sender, incoming is set to other
                context.msg_sender(),
                other
            )
        );

        let event1 = ExampleEvent1 { value2: AztecAddress::from_field(preimages[2]), value3: preimages[3] as u8 };

        event1.emit(
            encode_and_encrypt_event_with_randomness(
                &mut context,
                randomness[1],
                // outgoing is set to other, incoming is set to msg sender
                other,
                context.msg_sender()
            )
        );
    }

    #[aztec(public)]
    fn emit_unencrypted_events(preimages: [Field; 4]) {
        let event0 = ExampleEvent0 { value0: preimages[0], value1: preimages[1] };

        event0.emit(encode_event(&mut context));

        let event1 = ExampleEvent1 { value2: AztecAddress::from_field(preimages[2]), value3: preimages[3] as u8 };

        event1.emit(encode_event(&mut context));
    }
}
