import { AztecAddress } from '@aztec/circuits.js';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class ContractInstanceStore {
  #contractInstances: AztecMap<string, Buffer>;

  constructor(db: AztecKVStore) {
    this.#contractInstances = db.openMap('archiver_contract_instances');
  }

  addContractInstance(contractInstance: ContractInstanceWithAddress): Promise<boolean> {
    return this.#contractInstances.set(
      contractInstance.address.toString(),
      new SerializableContractInstance(contractInstance).toBuffer(),
    );
  }

  getContractInstance(address: AztecAddress): ContractInstanceWithAddress | undefined {
    const contractInstance = this.#contractInstances.get(address.toString());
    return contractInstance && SerializableContractInstance.fromBuffer(contractInstance).withAddress(address);
  }
}
