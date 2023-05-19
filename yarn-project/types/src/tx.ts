import { CircuitsWasm, KernelCircuitPublicInputs, Proof, PublicCallRequest, SignedTxRequest } from '@aztec/circuits.js';
import { computeTxHash } from '@aztec/circuits.js/abis';

import { TxHash } from './tx_hash.js';
import { UnverifiedData } from './unverified_data.js';
import { EncodedContractFunction } from './contract_data.js';
import { arrayNonEmptyLength } from '@aztec/foundation/collection';

/**
 * Defines valid fields for a private transaction.
 */
type PrivateTxFields = 'data' | 'proof' | 'unverifiedData';

/**
 * Defines valid fields for a public transaction.
 */
type PublicTxFields = 'txRequest';

/**
 * Defines private tx type.
 */
export type PrivateTx = Required<Pick<Tx, PrivateTxFields>> & Tx;

/**
 * Defines public tx type.
 */
export type PublicTx = Required<Pick<Tx, PublicTxFields>> & Tx;

/**
 * Checks if a tx is public.
 */
export function isPublicTx(tx: Tx): tx is PublicTx {
  return !!tx.txRequest;
}

/**
 * Checks if a tx is private.
 */
export function isPrivateTx(tx: Tx): tx is PrivateTx {
  return !!tx.data && !!tx.proof && !!tx.unverifiedData;
}

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private txHash?: Promise<TxHash>;

  /**
   * Creates a new private transaction.
   * @param data - Public inputs of the private kernel circuit.
   * @param proof - Proof from the private kernel circuit.
   * @param unverifiedData - Unverified data created by this tx.
   * @param newContractPublicFunctions - Public functions made available by this tx.
   * @param enqueuedPublicFunctionCalls - Preimages of the public call stack of the kernel output.
   * @returns A new private tx instance.
   */
  public static createPrivate(
    data: KernelCircuitPublicInputs,
    proof: Proof,
    unverifiedData: UnverifiedData,
    newContractPublicFunctions: EncodedContractFunction[],
    enqueuedPublicFunctionCalls: PublicCallRequest[],
  ): PrivateTx {
    return new this(
      data,
      proof,
      unverifiedData,
      undefined,
      newContractPublicFunctions,
      enqueuedPublicFunctionCalls,
    ) as PrivateTx;
  }

  /**
   * Creates a new public transaction from the given tx request.
   * @param txRequest - The tx request.
   * @returns New public tx instance.
   */
  public static createPublic(txRequest: SignedTxRequest): PublicTx {
    return new this(undefined, undefined, undefined, txRequest) as PublicTx;
  }

  /**
   * Creates a new transaction containing both private and public calls.
   * @param data - Public inputs of the private kernel circuit.
   * @param proof - Proof from the private kernel circuit.
   * @param unverifiedData - Unverified data created by this tx.
   * @param txRequest - The tx request defining the public call.
   * @returns A new tx instance.
   */
  public static createPrivatePublic(
    data: KernelCircuitPublicInputs,
    proof: Proof,
    unverifiedData: UnverifiedData,
    txRequest: SignedTxRequest,
  ): PrivateTx & PublicTx {
    return new this(data, proof, unverifiedData, txRequest) as PrivateTx & PublicTx;
  }

  /**
   * Creates a new transaction from the given tx request.
   * @param data - Public inputs of the private kernel circuit.
   * @param proof - Proof from the private kernel circuit.
   * @param unverifiedData - Unverified data created by this tx.
   * @param txRequest - The tx request defining the public call.
   * @returns A new tx instance.
   */
  public static create(
    data?: KernelCircuitPublicInputs,
    proof?: Proof,
    unverifiedData?: UnverifiedData,
    txRequest?: SignedTxRequest,
  ): Tx {
    return new this(data, proof, unverifiedData, txRequest);
  }

  /**
   * Checks if a tx is private.
   * @returns True if the tx is private, false otherwise.
   */
  public isPrivate(): this is PrivateTx {
    return isPrivateTx(this);
  }

  /**
   * Checks if a tx is public.
   * @returns True if the tx is public, false otherwise.
   */
  public isPublic(): this is PublicTx {
    return isPublicTx(this);
  }

  protected constructor(
    /**
     * Output of the private kernel circuit for this tx.
     */
    public readonly data?: KernelCircuitPublicInputs,
    /**
     * Proof from the private kernel circuit.
     */
    public readonly proof?: Proof,
    /**
     * Information not needed to verify the tx (e.g. Encrypted note pre-images etc.).
     */
    public readonly unverifiedData?: UnverifiedData,
    /**
     * Signed public function call data.
     */
    public readonly txRequest?: SignedTxRequest,
    /**
     * New public functions made available by this tx.
     */
    public readonly newContractPublicFunctions?: EncodedContractFunction[],
    /**
     * Enqueued public functions from the private circuit to be run by the sequencer.
     * Preimages of the public call stack entries from the private kernel circuit output.
     */
    public readonly enqueuedPublicFunctionCalls?: PublicCallRequest[],
  ) {
    const kernelPublicCallStackSize =
      data?.end.publicCallStack && arrayNonEmptyLength(data.end.publicCallStack, item => item.isZero());
    if (kernelPublicCallStackSize && kernelPublicCallStackSize > (enqueuedPublicFunctionCalls?.length ?? 0)) {
      throw new Error(
        `Missing preimages for enqueued public function calls in kernel circuit public inputs (expected ${kernelPublicCallStackSize}, got ${enqueuedPublicFunctionCalls?.length})`,
      );
    }
  }

  /**
   * Construct & return transaction hash.
   * @returns The transaction's hash.
   */
  getTxHash(): Promise<TxHash> {
    if (this.isPrivate()) {
      // Private kernel functions are executed client side and for this reason tx hash is already set as first nullifier
      const firstNullifier = this.data?.end.newNullifiers[0];
      return Promise.resolve(new TxHash(firstNullifier.toBuffer()));
    }

    if (this.isPublic()) {
      if (!this.txHash) this.txHash = getTxHashFromRequest(this.txRequest);
      return this.txHash;
    }

    throw new Error('Tx data incorrectly set.');
  }

  /**
   * Convenience function to get array of hashes for an array of txs.
   * @param txs - The txs to get the hashes from.
   * @returns The corresponding array of hashes.
   */
  static async getHashes(txs: Tx[]): Promise<TxHash[]> {
    return await Promise.all(txs.map(tx => tx.getTxHash()));
  }
}

/**
 * Calculates the hash based on a SignedTxRequest.
 * @param txRequest - The SignedTxRequest.
 * @returns The tx hash.
 */
async function getTxHashFromRequest(txRequest: SignedTxRequest) {
  return new TxHash(computeTxHash(await CircuitsWasm.get(), txRequest).toBuffer());
}
