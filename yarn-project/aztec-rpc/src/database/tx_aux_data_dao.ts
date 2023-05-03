import { AztecAddress, Fr } from '@aztec/circuits.js';
import { NotePreimage } from '../aztec_rpc_server/tx_aux_data/index.js';
import { Point } from '@aztec/foundation/fields';

export interface TxAuxDataDao {
  // Properties from the encrypted note
  contractAddress: AztecAddress;
  storageSlot: Fr;
  notePreimage: NotePreimage;
  // Computed properties
  nullifier: Fr;
  // The location in the tree
  index: bigint;
  // The public key that was used to encrypt the data
  account: Point;
}
