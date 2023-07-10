import { PrivateHistoricTreeRoots, TxContext } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
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
 * The execution context for a client tx simulation.
 */
export class ClientTxExecutionContext {
  private readRequestCommitmentIndices: bigint[] = [];

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
   * Create context for nested executions.
   * @returns ClientTxExecutionContext
   */
  public extend() {
    return new ClientTxExecutionContext(this.db, this.txContext, this.historicRoots, this.packedArgsCache);
  }

  /**
   * For getting accumulated data.
   * @returns An array of readRequestCommitment indices.
   */
  public getReadRequestCommitmentIndices() {
    return this.readRequestCommitmentIndices;
  }

  /**
   * Gets the notes for a contract address and storage slot.
   * Returns note preimages and their indices in the private data tree.
   * Note that indices are not passed to app circuit. They forwarded to
   * the kernel prover which uses them to compute witnesses to pass
   * to the private kernel.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param noteSize - The note size.
   * @param sortBy - The sort by fields.
   * @param sortOrder - The sort order fields.
   * @param limit - The limit.
   * @param offset - The offset.
   * @param returnSize - The return size.
   * @returns An array of ACVM fields for the note count and the requested note preimages,
   * and another array of indices corresponding to each note
   */
  public async getNotes(
    contractAddress: AztecAddress,
    storageSlot: ACVMField,
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    limit: ACVMField,
    offset: ACVMField,
    returnSize: ACVMField,
  ) {
    const { count, notes } = await this.db.getNotes(
      contractAddress,
      fromACVMField(storageSlot),
      sortBy.map(field => +field),
      sortOrder.map(field => +field),
      +limit,
      +offset,
    );
    const preimages = notes.flatMap(({ preimage }) => preimage);

    // TODO(dbanks12): https://github.com/AztecProtocol/aztec-packages/issues/779
    // if preimages length is > rrcIndices length, we are either relying on
    // the app circuit to remove fake preimages, or on the kernel to handle
    // the length diff.
    const indices = notes.map(({ index }) => index).filter(index => index != BigInt(-1));
    this.readRequestCommitmentIndices.push(...indices);

    const paddedZeros = Array(+returnSize - 1 - preimages.length).fill(Fr.ZERO);
    return [count, preimages, paddedZeros].flat().map(f => toACVMField(f));
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
    this.readRequestCommitmentIndices.push(commitmentInputs.index);
    return toAcvmCommitmentLoadOracleInputs(commitmentInputs, this.historicRoots.privateDataTreeRoot);
  }
}
