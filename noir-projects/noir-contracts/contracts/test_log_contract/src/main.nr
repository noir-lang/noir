contract TestLog {
    use dep::aztec::prelude::PrivateSet;
    use dep::aztec::protocol_types::{traits::Serialize, grumpkin_point::GrumpkinPoint, grumpkin_private_key::GrumpkinPrivateKey};
    use dep::value_note::value_note::ValueNote;
    use dep::aztec::encrypted_logs::incoming_body::EncryptedLogIncomingBody;
    use dep::aztec::event::event_interface::EventInterface;

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
    fn emit_encrypted_log(randomness: Field, event_type_id: Field, preimage: [Field; 6]) {
        let header = context.get_header();
        let msg_sender_ivpk_m = header.get_ivpk_m(&mut context, context.msg_sender());
        let msg_sender_ovpk_m = header.get_ovpk_m(&mut context, context.msg_sender());

        context.encrypt_and_emit_event(
            randomness,
            event_type_id,
            msg_sender_ovpk_m,
            msg_sender_ivpk_m,
            preimage
        );
    }

    #[aztec(private)]
    fn emit_encrypted_events(randomness: [Field; 2], preimages: [Field; 4]) {
        let header = context.get_header();
        let msg_sender_ivpk_m = header.get_ivpk_m(&mut context, context.msg_sender());
        let msg_sender_ovpk_m = header.get_ovpk_m(&mut context, context.msg_sender());

        let event0 = ExampleEvent0 { value0: preimages[0], value1: preimages[1] };

        context.encrypt_and_emit_event(
            randomness[0],
            ExampleEvent0::get_event_type_id().to_field(),
            msg_sender_ovpk_m,
            msg_sender_ivpk_m,
            event0.serialize()
        );

        let event1 = ExampleEvent1 { value2: preimages[2], value3: preimages[3] };

        context.encrypt_and_emit_event(
            randomness[1],
            ExampleEvent1::get_event_type_id().to_field(),
            msg_sender_ovpk_m,
            msg_sender_ivpk_m,
            event1.serialize()
        );
    }
}
