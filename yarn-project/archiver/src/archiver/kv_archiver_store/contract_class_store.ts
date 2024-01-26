import { Fr } from '@aztec/foundation/fields';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { ContractClassWithId, SerializableContractClass } from '@aztec/types/contracts';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class ContractClassStore {
  #contractClasses: AztecMap<string, Buffer>;

  constructor(db: AztecKVStore) {
    this.#contractClasses = db.openMap('archiver_contract_classes');
  }

  addContractClass(contractClass: ContractClassWithId): Promise<boolean> {
    return this.#contractClasses.set(
      contractClass.id.toString(),
      new SerializableContractClass(contractClass).toBuffer(),
    );
  }

  getContractClass(id: Fr): ContractClassWithId | undefined {
    const contractClass = this.#contractClasses.get(id.toString());
    return contractClass && SerializableContractClass.fromBuffer(contractClass).withId(id);
  }
}
