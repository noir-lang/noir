import { AztecAddress, Fr } from '@aztec/circuits.js';
import { NotePreimage } from '../aztec_rpc_server/tx_aux_data/note_preimage.js';

export interface NoteDao {
  // Properties from the encrypted note
  contractAddress: AztecAddress;
  contractSlot: Fr;
  notePreimage: NotePreimage;
  // Computed properties
  nullifier: Fr;
  // The location in the tree
  index: number;
}
