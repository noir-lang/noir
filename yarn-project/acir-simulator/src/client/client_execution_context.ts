import { PrivateHistoricTreeRoots, TxContext } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import {
  ACVMField,
  createDummyNote,
  fromACVMField,
  toACVMField,
  toAcvmMessageLoadOracleInputs,
} from '../acvm/index.js';
import { NoteLoadOracleInputs, DBOracle } from './db_oracle.js';

/**
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  constructor(
    /**  The database oracle. */
    public db: DBOracle,
    /** The tx context. */
    public txContext: TxContext,
    /** The old roots. */
    public historicRoots: PrivateHistoricTreeRoots,
  ) {}

  /**
   * Gets the notes for a contract address and storage slot.
   * Returns note preimages and their indices in the private data tree.
   * Note that indices are not passed to app circuit. They forwarded to
   * the kernel prover which uses them to compute witnesses to pass
   * to the private kernel.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param limit - The amount of notes to get.
   * @returns An array of ACVM fields for the note count and the requested note preimages,
   * and another array of indices corresponding to each note
   */
  public async getNotes(contractAddress: AztecAddress, storageSlot: ACVMField, limit: number) {
    const { count, notes } = await this.fetchNotes(contractAddress, storageSlot, limit);

    const preimages = [
      toACVMField(count),
      ...notes.flatMap(noteGetData => noteGetData.preimage.map(f => toACVMField(f))),
    ];
    const indices = notes.map(noteGetData => noteGetData.index);

    return { preimages, indices };
  }

  /**
   * Views the notes for a contract address and storage slot.
   * Doesn't include the leaf indices.
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

    const dummyNotes = Array.from(
      { length: Math.max(0, limit - notes.length) },
      (): NoteLoadOracleInputs => ({
        preimage: createDummyNote(),
        index: BigInt(-1),
      }),
    );

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
