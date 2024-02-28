use dep::protocol_types::{
  abis::function_selector::FunctionSelector,
  address::{AztecAddress, EthAddress},
};

trait ContextInterface {
    fn push_new_note_hash(&mut self, note_hash: Field);
    fn push_new_nullifier(&mut self, nullifier: Field, nullified_commitment: Field);
    fn msg_sender(self) -> AztecAddress;
    fn this_address(self) -> AztecAddress;
    fn this_portal_address(self) -> EthAddress;
    fn chain_id(self) -> Field;
    fn version(self) -> Field;
    fn selector(self) -> FunctionSelector;
}