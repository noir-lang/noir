import { AztecAddress, Fr } from '@aztec/foundation';
import { NotePreimage } from '../aztec_rpc_server/tx_aux_data/note_preimage.js';
import { TxAuxData } from '../aztec_rpc_server/tx_aux_data/tx_aux_data.js';

export class TxAuxDataDao {
  constructor(public notePreImage: NotePreimage, public contractAddress: AztecAddress, public storageSlot: Fr) {}

  public static fromTxAuxData(txAuxData: TxAuxData): TxAuxDataDao {
    return new TxAuxDataDao(txAuxData.notePreImage, txAuxData.contractAddress, txAuxData.storageSlot);
  }
}
