use crate::{
    context::PrivateContext, event::event_interface::EventInterface,
    encrypted_logs::payload::compute_encrypted_event_log, oracle::logs_traits::LensForEncryptedEvent
};
use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};

fn emit_with_keys<Event, NB, MB, OB>(
    context: &mut PrivateContext,
    randomness: Field,
    event: Event,
    ovpk: GrumpkinPoint,
    ivpk: GrumpkinPoint
) where Event: EventInterface<NB, MB>, [u8; NB]: LensForEncryptedEvent<NB, OB> {
    let contract_address: AztecAddress = context.this_address();
    let ovsk_app: Field  = context.request_ovsk_app(ovpk.hash());

    let encrypted_log: [u8; OB] = compute_encrypted_event_log(contract_address, randomness, ovsk_app, ovpk, ivpk, event);

    context.emit_raw_event_log_with_masked_address(randomness, encrypted_log);
}

pub fn encode_and_encrypt_event<Event, NB, MB, OB>(
    context: &mut PrivateContext,
    randomness: Field,
    ov: AztecAddress,
    iv: AztecAddress
) -> fn[(&mut PrivateContext, AztecAddress, AztecAddress, Field)](Event) -> () where Event: EventInterface<NB, MB>, [u8; NB]: LensForEncryptedEvent<NB, OB> {
    | e: Event | {
        let header = context.get_header();
        let ovpk = header.get_ovpk_m(context, ov);
        let ivpk = header.get_ivpk_m(context, iv);
        emit_with_keys(context, randomness, e, ovpk, ivpk);
    }
}

pub fn encode_and_encrypt_event_with_keys<Event, NB, MB, OB>(
    context: &mut PrivateContext,
    randomness: Field,
    ovpk: GrumpkinPoint,
    ivpk: GrumpkinPoint
) -> fn[(&mut PrivateContext, Field, GrumpkinPoint, GrumpkinPoint)](Event) -> () where Event: EventInterface<NB, MB>, [u8; NB]: LensForEncryptedEvent<NB, OB> {
    | e: Event | {
        emit_with_keys(context, randomness, e, ovpk, ivpk);
    }
}
