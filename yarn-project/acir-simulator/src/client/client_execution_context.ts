import { PrivateHistoricTreeRoots, TxContext } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import {
  ACVMField,
  createDummyNote,
  fromACVMField,
  toACVMField,
  toAcvmCommitmentLoadOracleInputs,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../acvm/index.js';
import { NoteLoadOracleInputs, DBOracle } from './db_oracle.js';
import { PackedArgsCache } from '../packed_args_cache.js';

/**
 * A type that wraps data with it's read request index
 */
type ACVMWithReadRequestIndex = {
  /** The index of the data in the tree. */
  index: bigint;
  /** The formatted data. */
  acvmData: ACVMField[];
};

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
    /** The cache of packed arguments */
    public packedArgsCache: PackedArgsCache,
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
   * @returns The l1 to l2 message data
   */
  public async getL1ToL2Message(msgKey: Fr): Promise<ACVMField[]> {
    const messageInputs = await this.db.getL1ToL2Message(msgKey);
    return toAcvmL1ToL2MessageLoadOracleInputs(messageInputs, this.historicRoots.l1ToL2MessagesTreeRoot);
  }

  /**
   * Fetches a path to prove existence of a commitment in the db, given its contract side commitment (before silo).
   * @param contractAddress - The contract address.
   * @param commitment - The commitment.
   * @returns The commitment data.
   */
  public async getCommitment(contractAddress: AztecAddress, commitment: Fr): Promise<ACVMWithReadRequestIndex> {
    const commitmentInputs = await this.db.getCommitmentOracle(contractAddress, commitment);
    return {
      acvmData: toAcvmCommitmentLoadOracleInputs(commitmentInputs, this.historicRoots.privateDataTreeRoot),
      index: commitmentInputs.index,
    };
  }
}
