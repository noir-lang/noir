use crate::context::PrivateContext;
use crate::note::note_header::NoteHeader;
use dep::protocol_types::{grumpkin_point::GrumpkinPoint, abis::event_selector::EventSelector};

trait EventInterface<NB, MB> {
    fn private_to_be_bytes(self, randomness: Field) -> [u8; NB];
    fn to_be_bytes(self) -> [u8; MB];
    fn get_event_type_id() -> EventSelector;
}
