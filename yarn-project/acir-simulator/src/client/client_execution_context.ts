import { PRIVATE_DATA_TREE_HEIGHT, PrivateHistoricTreeRoots } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { TxExecutionRequest } from '@aztec/types';
import {
  ACVMField,
  createDummyNote,
  fromACVMField,
  toACVMField,
  toAcvmMessageLoadOracleInputs,
  toAcvmNoteLoadOracleInputs,
} from '../acvm/index.js';
import { DBOracle } from './db_oracle.js';

/**
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  constructor(
    /**  The database oracle. */
    public db: DBOracle,
    /** The tx request. */
    public request: TxExecutionRequest,
    /** The old roots. */
    public historicRoots: PrivateHistoricTreeRoots,
  ) {}

  /**
   * Gets the notes for a contract address and storage slot.
   * Returns note load oracle inputs, which includes the paths and the roots.
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param limit - The amount of notes to get.
   * @returns The ACVM fields for the counts and the requested note load oracle inputs.
   */
  public async getNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number) {
    const { count, notes } = await this.fetchNotes(contractAddress, storageSlot, limit);
    return [
      toACVMField(count),
      ...notes.flatMap(noteGetData => toAcvmNoteLoadOracleInputs(noteGetData, this.historicRoots.privateDataTreeRoot)),
    ];
  }

  /**
   * Views the notes for a contract address and storage slot.
   * Doesn't include the sibling paths and the root.
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param limit - The amount of notes to get.
   * @param offset - The offset to start from (for pagination).
   * @returns The ACVM fields for the count and the requested notes.
   */
  public async viewNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number, offset = 0) {
    const { count, notes } = await this.fetchNotes(contractAddress, storageSlot, limit, offset);

    return [toACVMField(count), ...notes.flatMap(noteGetData => noteGetData.preimage.map(f => toACVMField(f)))];
  }

  /**
   * Fetches the notes for a contract address and storage slot from the db.
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param limit - The amount of notes to get.
   * @param offset - The offset to start from (for pagination).
   * @returns The count and the requested notes, padded with dummy notes.
   */
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

  /**
   * Fetches the a message from the db, given its key.
   * @param msgKey - A buffer representing the message key.
   * @returns The message data
   */
  public async getL1ToL2Message(msgKey: Fr): Promise<ACVMField[]> {
    const messageInputs = await this.db.getL1ToL2Message(msgKey);
    return toAcvmMessageLoadOracleInputs(messageInputs, this.historicRoots.l1ToL2MessagesTreeRoot);
  }
}
