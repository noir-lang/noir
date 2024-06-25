contract TestLog {
    use dep::aztec::prelude::PrivateSet;
    use dep::aztec::protocol_types::{traits::Serialize, grumpkin_point::GrumpkinPoint, grumpkin_private_key::GrumpkinPrivateKey};
    use dep::value_note::value_note::ValueNote;
    use dep::aztec::encrypted_logs::incoming_body::EncryptedLogIncomingBody;
    use dep::aztec::event::event_interface::EventInterface;
    use dep::aztec::encrypted_logs::encrypted_event_emission::{encode_and_encrypt_event, encode_and_encrypt_event_with_keys};

    #[aztec(event)]
    struct ExampleEvent0 {
        value0: Field,
        value1: Field,
    }

    #[aztec(event)]
    struct ExampleEvent1 {
        value2: Field,
        value3: Field,
    }

    #[aztec(storage)]
    struct Storage {
        example_set: PrivateSet<ValueNote>,
    }

    // EXAMPLE_EVENT_0_BYTES_LEN + 16
    global EXAMPLE_EVENT_0_CIPHERTEXT_BYTES_LEN = 144;

    #[aztec(private)]
    fn compute_incoming_log_body_ciphertext(
        secret: GrumpkinPrivateKey,
        point: GrumpkinPoint,
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
    fn emit_encrypted_events(randomness: [Field; 2], preimages: [Field; 4]) {
        let event0 = ExampleEvent0 { value0: preimages[0], value1: preimages[1] };

        event0.emit(
            encode_and_encrypt_event(
                &mut context,
                randomness[0],
                context.msg_sender(),
                context.msg_sender()
            )
        );

        let event1 = ExampleEvent1 { value2: preimages[2], value3: preimages[3] };

        event1.emit(
            encode_and_encrypt_event(
                &mut context,
                randomness[1],
                context.msg_sender(),
                context.msg_sender()
            )
        );
    }
}
