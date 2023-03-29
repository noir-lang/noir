import { AztecAddress, Fr } from '@aztec/circuits.js';
import { NotePreImage } from '../aztec_rpc_server/note_preimage/note_preimage.js';

export interface NoteDao {
  // Properties from the encrypted note
  contractAddress: AztecAddress;
  contractSlot: Fr;
  notePreimage: NotePreImage;
  // Computed properties
  nullifier: Fr;
  // The location in the tree
  index: number;
}
