use crate::{
    context::{PrivateContext, PublicContext}, event::event_interface::EventInterface,
    encrypted_logs::payload::compute_encrypted_event_log, oracle::logs_traits::LensForEncryptedEvent
};
use dep::protocol_types::{address::AztecAddress, traits::Serialize};

fn emit<Event, NB, MB, OB, N, M>(
    context: &mut PublicContext,
    event: Event
) where Event: EventInterface<NB, MB>, Event: Serialize<N>, [Field; N]: LensForEventSelector<N, M> {
    let selector = Event::get_event_type_id();

    let serialized_event = event.serialize();
    let mut emitted_log = [0; M];

    // We put the selector in the "last" place, to avoid reading or assigning to an expression in an index
    for i in 0..serialized_event.len() {
        emitted_log[i] = serialized_event[i];
    }

    emitted_log[serialized_event.len()] = selector.to_field();

    context.emit_unencrypted_log(emitted_log);
}

pub fn encode_event<Event, NB, MB, OB, N, M>(context: &mut PublicContext) -> fn[(&mut PublicContext,)](Event) -> () where Event: EventInterface<NB, MB>, Event: Serialize<N>, [Field; N]: LensForEventSelector<N, M> {
    | e: Event | {
        emit(
          context,
          e,
        );
    }
}

trait LensForEventSelector<N, M> {
    // N = event preimage input in fields
    // M = event preimage input in fields + event selector as field
    fn output(self: [Field; N]) -> [Field; M];
}

impl LensForEventSelector<1, 2> for [Field; 1] {
    fn output(self) -> [Field; 2] {[self[0] as Field; 2]}
}
impl LensForEventSelector<2, 3> for [Field; 2] {
    fn output(self) -> [Field; 3] {[self[0] as Field; 3]}
}
impl LensForEventSelector<3, 4> for [Field; 3] {
    fn output(self) -> [Field; 4] {[self[0] as Field; 4]}
}
impl LensForEventSelector<4, 5> for [Field; 4] {
    fn output(self) -> [Field; 5] {[self[0] as Field; 5]}
}
impl LensForEventSelector<5, 6> for [Field; 5] {
    fn output(self) -> [Field; 6] {[self[0] as Field; 6]}
}
impl LensForEventSelector<6, 7> for [Field; 6] {
    fn output(self) -> [Field; 7] {[self[0] as Field; 7]}
}
