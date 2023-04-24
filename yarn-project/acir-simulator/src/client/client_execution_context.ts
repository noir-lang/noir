import { ACVMField, toACVMField, fromACVMField, toAcvmNoteLoadOracleInputs, createDummyNote } from '../acvm/index.js';
import { AztecAddress, Fr } from '@aztec/foundation';
import { PrivateHistoricTreeRoots, TxRequest, PRIVATE_DATA_TREE_HEIGHT } from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';

export class ClientTxExecutionContext {
  constructor(public db: DBOracle, public request: TxRequest, public historicRoots: PrivateHistoricTreeRoots) {}

  public async getNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number) {
    const { count, notes } = await this.fetchNotes(contractAddress, storageSlot, limit);
    return [
      toACVMField(count),
      ...notes.flatMap(noteGetData => toAcvmNoteLoadOracleInputs(noteGetData, this.historicRoots.privateDataTreeRoot)),
    ];
  }

  public async viewNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number, offset = 0) {
    const { count, notes } = await this.fetchNotes(contractAddress, storageSlot, limit, offset);

    return [toACVMField(count), ...notes.flatMap(noteGetData => noteGetData.preimage.map(f => toACVMField(f)))];
  }

  private async fetchNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number, offset = 0) {
    const { count, notes } = await this.db.getNotes(contractAddress, fromACVMField(storageSlot), limit, offset);

    const dummyNotes = Array.from({ length: Math.max(0, limit - notes.length) }, () => ({
      preimage: createDummyNote(),
      siblingPath: new Array(PRIVATE_DATA_TREE_HEIGHT).fill(Fr.ZERO),
      index: 0n,
    }));

    return {
      count,
      notes: notes.concat(dummyNotes),
    };
  }
}
