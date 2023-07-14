import { PrivateHistoricTreeRoots, ReadRequestMembershipWitness, TxContext } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';

import {
  ACVMField,
  fromACVMField,
  toACVMField,
  toAcvmCommitmentLoadOracleInputs,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../acvm/index.js';
import { PackedArgsCache } from '../packed_args_cache.js';
import { DBOracle } from './db_oracle.js';

/**
 * Information about a note created during execution.
 */
export type PendingNoteData = {
  /** The preimage of the created note */
  preimage: ACVMField[];
  /** The contract address of the commitment. */
  contractAddress: AztecAddress;
  /** The storage slot of the commitment. */
  storageSlot: Fr;
};

/**
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  // Note: not forwarded to nested contexts via `extend()` because these witnesses
  // are meant to be maintained on a per-call basis as they should mirror read requests
  // output by an app circuit via public inputs.
  private readRequestPartialWitnesses: ReadRequestMembershipWitness[] = [];

  constructor(
    /**  The database oracle. */
    public db: DBOracle,
    /** The tx context. */
    public txContext: TxContext,
    /** The old roots. */
    public historicRoots: PrivateHistoricTreeRoots,
    /** The cache of packed arguments */
    public packedArgsCache: PackedArgsCache,
    /** Pending commitments created (and not nullified) up to current point in execution **/
    public pendingNotes: PendingNoteData[] = [],
  ) {}

  /**
   * Create context for nested executions.
   * @returns ClientTxExecutionContext
   */
  public extend() {
    return new ClientTxExecutionContext(
      this.db,
      this.txContext,
      this.historicRoots,
      this.packedArgsCache,
      this.pendingNotes,
    );
  }

  /**
   * For getting accumulated data.
   * @returns An array of partially filled in read request membership witnesses.
   */
  public getReadRequestPartialWitnesses() {
    return this.readRequestPartialWitnesses;
  }

  /**
   * For getting secret key.
   * @param contractAddress - The contract address.
   * @param ownerX - The x coordinate of the owner's public key.
   * @param ownerY - The y coordinate of the owner's public key.
   * @returns The secret key of the owner.
   */
  public async getSecretKey(contractAddress: AztecAddress, ownerX: ACVMField, ownerY: ACVMField) {
    return toACVMField(
      await this.db.getSecretKey(contractAddress, new Point(fromACVMField(ownerX), fromACVMField(ownerY))),
    );
  }

  /**
   * Gets some notes for a contract address and storage slot.
   * Returns a flattened array containing real-note-count and note preimages.
   *
   * @remarks
   *
   * Check for pending notes with matching address/slot.
   * If limit isn't reached after pending notes are checked/retrieved,
   * fetchNotes from DB with modified limit.
   * Real notes coming from DB will have a leafIndex which
   * represents their index in the private data tree.
   *
   * This function will populate this.readRequestPartialWitnesses which
   * here is just used to flag reads as "transient" (one in getPendingNotes)
   * or to flag non-transient reads with their leafIndex.
   * The KernelProver will use this to fully populate witnesses and provide hints to
   * the kernel regarding which commitments each transient read request corresponds to.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param sortBy - The sort by fields.
   * @param sortOrder - The sort order fields.
   * @param limit - The limit.
   * @param offset - The offset.
   * @param returnSize - The return size.
   * @returns Flattened array of ACVMFields (format expected by Noir/ACVM) containing:
   * count - number of real (non-padding) notes retrieved,
   * preimages - the real note preimages retrieved, and
   * paddedZeros - zeros to ensure an array with length returnSize expected by Noir circuit
   */
  public async getNotes(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    limit: ACVMField,
    offset: ACVMField,
    returnSize: ACVMField,
  ): Promise<ACVMField[]> {
    const { pendingCount, pendingPreimages } = this.getPendingNotes(
      contractAddress,
      storageSlot,
      sortBy,
      sortOrder,
      limit,
      offset,
    );

    const dbLimit = +limit - pendingCount;
    const { count: dbCount, notes: dbNotes } = await this.db.getNotes(
      contractAddress,
      fromACVMField(storageSlot),
      sortBy.map(field => +field),
      sortOrder.map(field => +field),
      dbLimit,
      +offset,
    );
    // Noir (ACVM) expects a flattened (basically serialized) array of ACVMFields
    const dbPreimages = dbNotes.flatMap(({ preimage }) => preimage).map(f => toACVMField(f));

    // Combine pending and db preimages into a single flattened array.
    const preimages = [...pendingPreimages, ...dbPreimages];

    // Add a partial witness for each note from the db containing only the note index.
    // By default they will be flagged as non-transient.
    this.readRequestPartialWitnesses.push(...dbNotes.map(note => ReadRequestMembershipWitness.empty(note.index)));

    const paddedZeros = Array(+returnSize - 2 - preimages.length).fill(toACVMField(Fr.ZERO));
    return [toACVMField(pendingCount + dbCount), toACVMField(contractAddress), ...preimages, ...paddedZeros];
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
  public async getCommitment(contractAddress: AztecAddress, commitment: ACVMField) {
    const commitmentInputs = await this.db.getCommitmentOracle(contractAddress, fromACVMField(commitment));
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1029): support pending commitments here
    this.readRequestPartialWitnesses.push(ReadRequestMembershipWitness.empty(commitmentInputs.index));
    return toAcvmCommitmentLoadOracleInputs(commitmentInputs, this.historicRoots.privateDataTreeRoot);
  }

  /**
   * Gets some pending notes for a contract address and storage slot.
   * Returns number of notes retrieved and a flattened array of fields representing pending notes.
   *
   * Details:
   * Check for pending notes with matching address/slot.
   * Pending notes will have no leaf index and will be flagged
   * as transient since they don't exist (yet) in the private data tree.
   *
   * This function will partially populate this.readRequestPartialWitnesses solely
   * to flag these reads as "transient" since they correspond to pending commitments.
   * The KernelProver will use this to fill in hints to the kernel regarding which
   * commitments each transient read request corresponds to.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param _sortBy - The sort by fields.
   * @param _sortOrder - The sort order fields.
   * @param limit - The limit.
   * @param _offset - The offset.
   * @returns pendingCount - number of pending notes retrieved, and
   * pendingPreimages - flattened array of ACVMFields (format expected by Noir/ACVM)
   * containing the retrieved note preimages
   */
  private getPendingNotes(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    _sortBy: ACVMField[],
    _sortOrder: ACVMField[],
    limit: ACVMField,
    _offset: ACVMField,
  ) {
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/920): don't 'get' notes nullified in pendingNullifiers
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1030): enforce sorting and offset for pending notes
    let pendingCount = 0;
    // Noir (ACVM) expects a flattened (basically serialized) array of ACVMFields
    const pendingPreimages: ACVMField[] = []; // flattened fields representing preimages
    for (const note of this.pendingNotes) {
      if (pendingCount == +limit) {
        break;
      }
      if (note.contractAddress.equals(contractAddress) && note.storageSlot.equals(fromACVMField(storageSlot))) {
        pendingCount++;
        pendingPreimages.push(...note.preimage); // flattened
        this.readRequestPartialWitnesses.push(ReadRequestMembershipWitness.emptyTransient());
      }
    }
    return { pendingCount, pendingPreimages };
  }
}
