use dep::std::option::Option;
use dep::aztec::constants_gen::MAX_READ_REQUESTS_PER_CALL;
use crate::address_note::AddressNote;

fn filter_by_owner_and_address(
    notes: [Option<AddressNote>; MAX_READ_REQUESTS_PER_CALL],
    params: [Field; 2],
) -> [Option<AddressNote>; 1] {
    let address = params[0];
    let owner = params[1];
    let mut owner_note = [Option::none()];
    for i in 0..notes.len() {
        if notes[i].is_some() {
            let note = notes[i].unwrap_unchecked();
            if (note.address == address) & (note.owner == owner) {
                owner_note[0] = notes[i];
            }
        }
    }
    owner_note
}
