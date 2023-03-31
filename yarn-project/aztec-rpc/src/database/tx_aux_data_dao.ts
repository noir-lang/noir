import { AztecAddress, Fr } from '@aztec/circuits.js';
import { NotePreimage } from '../aztec_rpc_server/tx_aux_data/index.js';

export interface TxAuxDataDao {
  // Properties from the encrypted note
  contractAddress: AztecAddress;
  storageSlot: Fr;
  notePreimage: NotePreimage;
  // Computed properties
  nullifier: Fr;
  // The location in the tree
  index: number;
}
