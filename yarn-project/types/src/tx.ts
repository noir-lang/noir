import { CircuitsWasm, PrivateKernelPublicInputs, UInt8Vector } from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { createTxHash } from './create_tx_hash.js';
import { TxHash } from './tx_hash.js';
import { UnverifiedData } from './unverified_data.js';

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private hashPromise?: Promise<TxHash>;

  /**
   *
   * @param data - Tx inputs.
   * @param proof - Tx proof.
   * @param unverifiedData  - Information not needed to verify the tx (e.g. encrypted note pre-images etc.)
   * @param isEmpty - Whether this is a placeholder empty tx.
   */
  constructor(
    public readonly data: PrivateKernelPublicInputs,
    public readonly proof: UInt8Vector,
    public readonly unverifiedData: UnverifiedData,
    public readonly isEmpty = false,
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
   * Utility function to generate tx hash.
   * @param tx - The transaction from which to generate the hash.
   * @returns A hash of the tx data that identifies the tx.
   */
  static async createTxHash(tx: Tx): Promise<TxHash> {
    // NOTE: We are using computeContractLeaf here to ensure consistency with how circuits compute
    // contract tree leaves, which then go into the L2 block, which are then used to regenerate
    // the tx hashes. This means we need the full circuits wasm, and cannot use the lighter primitives
    // wasm. Alternatively, we could stop using computeContractLeaf and manually use the same
    const wasm = await CircuitsWasm.get();
    return createTxHash({
      ...tx.data.end,
      newContracts: tx.data.end.newContracts.map(cd => computeContractLeaf(wasm, cd)),
    });
  }
}
