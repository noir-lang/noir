mod private_token_airdrop_interface;

// Demonstrates how to perform 4 x 4 = 16 transfers in one transaction. Uses the private airdrop contract in the backend.
contract MultiTransfer {
    use dep::aztec::oracle::public_call;
    use dep::aztec::private_call_stack_item::PrivateCallStackItem;
    use dep::aztec::public_call_stack_item::PublicCallStackItem;
    use dep::aztec::types::point::Point;

    // Libs
    use dep::value_note::{
        value_note::{VALUE_NOTE_LEN, ValueNoteMethods},
    };
    use dep::aztec::note::{
        note_header::NoteHeader,
        utils as note_utils,
    };

    // Interfaces
    use crate::private_token_airdrop_interface::PrivateTokenAirdropPrivateContextInterface;

    #[aztec(private)]
    fn constructor() {}

    // Transfers 12 amounts to 12 recipients.
    // multiTransfer() => 4 calls to batchTransfer() on the private airdrop contract.
    // Each batchTransfer() call allows sending new notes to 3 recipients, so 3 x 4 = 12 recipients in total.
    // Note that all the notes stay on the airdrop contract, the multi transfer contract must interact with 
    // methods in the private airdrop contract to initiate multiple transfers in one transaction. 
    #[aztec(private)]
    fn multiTransfer(
        asset: Field, // Asset to distribute
        addresses: [Field; 12], // Addresses to distribute to
        amounts: [Field; 12], // Amounts to distribute
        owner: Field, // Owner of the asset
        note_offsets: [Field; 4], // Offsets from which 4 notes of the owner would be read.
    ) -> [Field; 4] {
        let token = PrivateTokenAirdropPrivateContextInterface::at(asset);

        // First batch transfer call
        let result1 = token.batchTransfer(
            &mut context,
            owner, 
            [amounts[0], amounts[1], amounts[2]],
            [addresses[0], addresses[1], addresses[2]],
            note_offsets[0] as u32,
        )[0];

        // Second batch transfer call
        let result2 = token.batchTransfer(
            &mut context,
            owner, 
            [amounts[3], amounts[4], amounts[5]],
            [addresses[3], addresses[4], addresses[5]],
            note_offsets[1] as u32,
        )[0];

        // Third batch transfer call
        let result3 = token.batchTransfer(
            &mut context,
            owner, 
            [amounts[6], amounts[7], amounts[8]],
            [addresses[6], addresses[7], addresses[8]],
            note_offsets[2] as u32,
        )[0];

        // Fourth batch transfer call
        let result4 = token.batchTransfer(
            &mut context,
            owner, 
            [amounts[9], amounts[10], amounts[11]],
            [addresses[9], addresses[10], addresses[11]],
            note_offsets[3] as u32,
        )[0];

        [result1, result2, result3, result4]
    }
}