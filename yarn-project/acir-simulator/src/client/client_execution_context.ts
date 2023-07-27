import { CircuitsWasm, PrivateHistoricTreeRoots, ReadRequestMembershipWitness, TxContext } from '@aztec/circuits.js';
import { computeCommitmentNonce } from '@aztec/circuits.js/abis';
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
import { DBOracle, PendingNoteData } from './db_oracle.js';
import { pickNotes } from './pick_notes.js';

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
    /** The tx nullifier, which is also the first nullifier. This will be used to compute the nonces for pending notes. */
    private txNullifier: Fr,
    /** The tx context. */
    public txContext: TxContext,
    /** The old roots. */
    public historicRoots: PrivateHistoricTreeRoots,
    /** The cache of packed arguments */
    public packedArgsCache: PackedArgsCache,
    /** Pending notes created (and not nullified) up to current point in execution.
     *  If a nullifier for a note in this list is emitted, the note will be REMOVED. */
    private pendingNotes: PendingNoteData[] = [],
    /** The list of nullifiers created in this transaction. The commitment/note which is nullified
     *  might be pending or not (i.e., was generated in a previous transaction) */
    private pendingNullifiers: Set<Fr> = new Set<Fr>(),
  ) {}

  /**
   * Create context for nested executions.
   * @returns ClientTxExecutionContext
   */
  public extend() {
    return new ClientTxExecutionContext(
      this.db,
      this.txNullifier,
      this.txContext,
      this.historicRoots,
      this.packedArgsCache,
      this.pendingNotes,
      this.pendingNullifiers,
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
      (await this.db.getSecretKey(contractAddress, new Point(fromACVMField(ownerX), fromACVMField(ownerY)))).value,
    );
  }

  /**
   * Gets some notes for a contract address and storage slot.
   * Returns a flattened array containing real-note-count and note preimages.
   *
   * @remarks
   *
   * Check for pending notes with matching address/slot.
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
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query.
   * @param offset - The starting index for pagination.
   * @param returnSize - The return size.
   * @returns Flattened array of ACVMFields (format expected by Noir/ACVM) containing:
   * count - number of real (non-padding) notes retrieved,
   * contractAddress - the contract address,
   * preimages - the real note preimages retrieved, and
   * paddedZeros - zeros to ensure an array with length returnSize expected by Noir circuit
   */
  public async getNotes(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    limit: number,
    offset: number,
    returnSize: number,
  ): Promise<ACVMField[]> {
    const storageSlotField = fromACVMField(storageSlot);

    const pendingNotes = this.pendingNotes.filter(
      n => n.contractAddress.equals(contractAddress) && n.storageSlot.equals(storageSlotField),
    );

    const dbNotes = await this.db.getNotes(contractAddress, storageSlotField);

    // Remove notes which were already nullified during this transaction.
    const dbNotesFiltered = dbNotes.filter(n => !this.pendingNullifiers.has(n.nullifier as Fr));

    // Nullified pending notes are already removed from the list.
    const notes = pickNotes([...dbNotesFiltered, ...pendingNotes], {
      sortBy: sortBy.map(field => +field),
      sortOrder: sortOrder.map(field => +field),
      limit,
      offset,
    });

    // Combine pending and db preimages into a single flattened array.
    const preimages = notes.flatMap(({ nonce, preimage }) => [nonce, ...preimage]);

    // Add a partial witness for each note.
    // It contains the note index for db notes. And flagged as transient for pending notes.
    notes.forEach(({ index }) => {
      this.readRequestPartialWitnesses.push(
        index !== undefined ? ReadRequestMembershipWitness.empty(index) : ReadRequestMembershipWitness.emptyTransient(),
      );
    });

    const paddedZeros = Array(Math.max(0, returnSize - 2 - preimages.length)).fill(Fr.ZERO);
    return [notes.length, contractAddress, ...preimages, ...paddedZeros].map(v => toACVMField(v));
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
   * Process new note created during execution.
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param preimage - new note preimage.
   * @param nullifier - note nullifier
   * @param innerNoteHash - inner note hash
   */
  public async pushNewNote(contractAddress: AztecAddress, storageSlot: Fr, preimage: Fr[], innerNoteHash: Fr) {
    const wasm = await CircuitsWasm.get();
    const nonce = computeCommitmentNonce(wasm, this.txNullifier, this.pendingNotes.length);
    this.pendingNotes.push({
      contractAddress,
      storageSlot: storageSlot,
      nonce,
      preimage,
      innerNoteHash,
    });
  }

  /**
   * Adding a nullifier into the current set of all pending nullifiers created
   * within the current transaction/execution.
   * @param nullifier - The pending nullifier to add in the list.
   */
  public pushPendingNullifier(nullifier: Fr) {
    this.pendingNullifiers.add(nullifier);
  }

  /**
   * Update the list of pending notes by chopping a note which is nullified.
   * The identifier used to determine matching is the inner note hash value.
   * However, we adopt a defensive approach and ensure that the contract address
   * and storage slot do match.
   * Note that this method might be called with an innerNoteHash referring to
   * a note created in a previous transaction which will result in this array
   * of pending notes left unchanged.
   * @param innerNoteHash - The inner note hash value to which note will be chopped.
   * @param contractAddress - The contract address
   * @param storageSlot - The storage slot as a Field Fr element
   */
  public nullifyPendingNotes(innerNoteHash: Fr, contractAddress: AztecAddress, storageSlot: Fr) {
    // IMPORTANT: We do need an in-place array mutation of this.pendingNotes as this array is shared
    // by reference among the nested calls. That is the main reason for 'splice' usage below.
    this.pendingNotes.splice(
      0,
      this.pendingNotes.length,
      ...this.pendingNotes.filter(
        n =>
          !(
            n.innerNoteHash.equals(innerNoteHash) &&
            n.contractAddress.equals(contractAddress) &&
            n.storageSlot.equals(storageSlot)
          ),
      ),
    );
  }
}
