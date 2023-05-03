import { CircuitsWasm, KernelCircuitPublicInputs, SignedTxRequest, UInt8Vector } from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';

import { createTxHash } from './create_tx_hash.js';
import { TxHash } from './tx_hash.js';
import { UnverifiedData } from './unverified_data.js';
import { EncodedContractFunction } from './contract_data.js';
import { keccak } from '@aztec/foundation/crypto';

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
  private hashPromise?: Promise<TxHash>;

  /**
   * Creates a new private transaction.
   * @param data - Public inputs of the private kernel circuit.
   * @param proof - Proof from the private kernel circuit.
   * @param unverifiedData - Unverified data created by this tx.
   * @param newContractPublicFunctions - Public functions made available by this tx.
   * @returns A new private tx instance.
   */
  public static createPrivate(
    data: KernelCircuitPublicInputs,
    proof: UInt8Vector,
    unverifiedData: UnverifiedData,
    newContractPublicFunctions?: EncodedContractFunction[],
  ): PrivateTx {
    return new this(data, proof, unverifiedData, undefined, newContractPublicFunctions) as PrivateTx;
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
    proof: UInt8Vector,
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
    proof?: UInt8Vector,
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
    public readonly proof?: UInt8Vector,
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
  ) {}

  /**
   * Construct & return transaction hash.
   * @returns The transaction's hash.
   */
  getTxHash(): Promise<TxHash> {
    if (!this.hashPromise) {
      this.hashPromise = Tx.createTxHash(this);
    }
    return this.hashPromise;
  }

  /**
   * Convenience function to get array of hashes for an array of txs.
   * @param txs - The txs to get the hashes from.
   * @returns The corresponding array of hashes.
   */
  static async getHashes(txs: Tx[]): Promise<TxHash[]> {
    return await Promise.all(txs.map(tx => tx.getTxHash()));
  }

  /**
   * Utility function to generate tx hash.
   * @param tx - The transaction from which to generate the hash.
   * @returns A hash of the tx data that identifies the tx.
   */
  static async createTxHash(tx: Tx): Promise<TxHash> {
    // TODO: Until we define how txs will be represented on the L2 block, we won't know how to
    // recreate their hash from the L2 block info. So for now we take the easy way out. If the
    // tx has only private data, we keep the same hash as before. If it has public data,
    // we hash it and return it. And if it has both, we compute both hashes
    // and hash them together. We'll probably want to change this later!
    // See https://github.com/AztecProtocol/aztec3-packages/issues/271
    const hashes = [];

    // NOTE: We are using computeContractLeaf here to ensure consistency with how circuits compute
    // contract tree leaves, which then go into the L2 block, which are then used to regenerate
    // the tx hashes. This means we need the full circuits wasm, and cannot use the lighter primitives
    // wasm. Alternatively, we could stop using computeContractLeaf and manually use the same hash.
    if (tx.data) {
      const wasm = await CircuitsWasm.get();
      hashes.push(
        createTxHash({
          ...tx.data.end,
          newContracts: tx.data.end.newContracts.map(cd => computeContractLeaf(wasm, cd)),
        }),
      );
    }

    // We hash the full signed tx request object (this is, the tx request along with the signature),
    // just like Ethereum does.
    if (tx.txRequest) {
      hashes.push(new TxHash(keccak(tx.txRequest.toBuffer())));
    }

    // Return a tx hash if we have only one, or hash them again if we have both
    if (hashes.length === 1) return hashes[0];
    else return new TxHash(keccak(Buffer.concat(hashes.map(h => h.buffer))));
  }
}
