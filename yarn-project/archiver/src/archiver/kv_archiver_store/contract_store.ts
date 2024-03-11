import { ExtendedContractData } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';

import { BlockStore } from './block_store.js';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class ContractStore {
  #blockStore: BlockStore;
  #extendedContractData: AztecMap<number, Buffer[]>;
  #log = createDebugLogger('aztec:archiver:contract_store');

  constructor(private db: AztecKVStore, blockStore: BlockStore) {
    this.#extendedContractData = db.openMap('archiver_extended_contract_data');
    this.#blockStore = blockStore;
  }

  /**
   * Add new extended contract data from an L2 block to the store's list.
   * @param data - List of contracts' data to be added.
   * @param blockNum - Number of the L2 block the contract data was deployed in.
   * @returns True if the operation is successful.
   */
  addExtendedContractData(data: ExtendedContractData[], blockNum: number): Promise<boolean> {
    return this.#extendedContractData.swap(blockNum, (existingData = []) => {
      existingData.push(...data.map(d => d.toBuffer()));
      return existingData;
    });
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): ExtendedContractData | undefined {
    const [blockNumber, _] = this.#blockStore.getContractLocation(contractAddress) ?? [];

    if (typeof blockNumber !== 'number') {
      return undefined;
    }

    for (const contract of this.#extendedContractData.get(blockNumber) ?? []) {
      const extendedContractData = ExtendedContractData.fromBuffer(contract);
      if (extendedContractData.contractData.contractAddress.equals(contractAddress)) {
        return extendedContractData;
      }
    }

    return undefined;
  }
}
