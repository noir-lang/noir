use dep::protocol_types::abis::event_selector::EventSelector;

trait EventInterface<NB, MB> {
    fn private_to_be_bytes(self, randomness: Field) -> [u8; NB];
    fn to_be_bytes(self) -> [u8; MB];
    fn get_event_type_id() -> EventSelector;
    fn emit<Env>(self, _emit: fn[Env](Self) -> ());
}
