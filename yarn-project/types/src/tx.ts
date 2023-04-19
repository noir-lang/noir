import { CircuitsWasm, PrivateKernelPublicInputs, SignedTxRequest, UInt8Vector } from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { createTxHash } from './create_tx_hash.js';
import { TxHash } from './tx_hash.js';
import { UnverifiedData } from './unverified_data.js';
import { keccak } from '@aztec/foundation';

export type PrivateTx = Required<Pick<Tx, 'data' | 'proof' | 'unverifiedData'>> & Tx;
export type PublicTx = Required<Pick<Tx, 'txRequest'>> & Tx;

export function isPublicTx(tx: Tx): tx is PublicTx {
  return tx.isPublic();
}

export function isPrivateTx(tx: Tx): tx is PrivateTx {
  return tx.isPrivate();
}

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private hashPromise?: Promise<TxHash>;

  public static createPrivate(
    data: PrivateKernelPublicInputs,
    proof: UInt8Vector,
    unverifiedData: UnverifiedData,
  ): PrivateTx {
    return new this(data, proof, unverifiedData, undefined) as PrivateTx;
  }

  public static createPublic(txRequest: SignedTxRequest): PublicTx {
    return new this(undefined, undefined, undefined, txRequest) as PublicTx;
  }

  public static createPrivatePublic(
    data: PrivateKernelPublicInputs,
    proof: UInt8Vector,
    unverifiedData: UnverifiedData,
    txRequest: SignedTxRequest,
  ): PrivateTx & PublicTx {
    return new this(data, proof, unverifiedData, txRequest) as PrivateTx & PublicTx;
  }

  public static create(
    data?: PrivateKernelPublicInputs,
    proof?: UInt8Vector,
    unverifiedData?: UnverifiedData,
    txRequest?: SignedTxRequest,
  ): Tx {
    const tx = new this(data, proof, unverifiedData, txRequest);
    if (!tx.isPrivate() && !tx.isPublic()) {
      throw new Error(`Tx needs either public or private data`);
    }
    return tx;
  }

  public isPrivate(): this is PrivateTx {
    return !!this.data && !!this.proof && !!this.unverifiedData;
  }

  public isPublic(): this is PublicTx {
    return !!this.txRequest;
  }

  /**
   * Creates a new instance.
   * @param data - Output of the private kernel circuit for this tx.
   * @param proof - Proof from the private kernel circuit.
   * @param unverifiedData  - Information not needed to verify the tx (e.g. encrypted note pre-images etc.)
   * @param txRequest - Signed public function call data.
   */
  protected constructor(
    public readonly data?: PrivateKernelPublicInputs,
    public readonly proof?: UInt8Vector,
    public readonly unverifiedData?: UnverifiedData,
    public readonly txRequest?: SignedTxRequest,
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
   * @param txs - the txs to get the hashes from
   * @returns The corresponding array of hashes
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
    if (tx.isPrivate()) {
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
    if (tx.isPublic()) {
      hashes.push(new TxHash(keccak(tx.txRequest.toBuffer())));
    }

    // Return a tx hash if we have only one, or hash them again if we have both
    if (hashes.length === 1) return hashes[0];
    else return new TxHash(keccak(Buffer.concat(hashes.map(h => h.buffer))));
  }
}
