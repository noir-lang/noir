// docs:start:prelude
use dep::protocol_types::{address::{AztecAddress, EthAddress}, abis::function_selector::FunctionSelector};
use crate::{
    state_vars::{
    map::Map, private_immutable::PrivateImmutable, private_mutable::PrivateMutable,
    public_immutable::PublicImmutable, public_mutable::PublicMutable, private_set::PrivateSet,
    shared_immutable::SharedImmutable
},
    log::{emit_unencrypted_log, emit_encrypted_log}, context::PrivateContext,
    note::{
    note_header::NoteHeader, note_interface::NoteInterface, note_getter_options::NoteGetterOptions,
    note_viewer_options::NoteViewerOptions,
    utils::compute_note_hash_and_nullifier as utils_compute_note_hash_and_nullifier
}
};
// docs:end:prelude
