import { Fr, FunctionSelector } from '@aztec/circuits.js';
import { BufferReader, numToUInt8, serializeToBuffer } from '@aztec/foundation/serialize';
import { AztecKVStore, AztecMap } from '@aztec/kv-store';
import { ContractClassPublic } from '@aztec/types/contracts';

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class ContractClassStore {
  #contractClasses: AztecMap<string, Buffer>;

  constructor(db: AztecKVStore) {
    this.#contractClasses = db.openMap('archiver_contract_classes');
  }

  addContractClass(contractClass: ContractClassPublic): Promise<boolean> {
    return this.#contractClasses.set(contractClass.id.toString(), serializeContractClassPublic(contractClass));
  }

  getContractClass(id: Fr): ContractClassPublic | undefined {
    const contractClass = this.#contractClasses.get(id.toString());
    return contractClass && { ...deserializeContractClassPublic(contractClass), id };
  }

  getContractClassIds(): Fr[] {
    return Array.from(this.#contractClasses.keys()).map(key => Fr.fromString(key));
  }
}

export function serializeContractClassPublic(contractClass: ContractClassPublic): Buffer {
  return serializeToBuffer(
    numToUInt8(contractClass.version),
    contractClass.artifactHash,
    contractClass.privateFunctions?.length ?? 0,
    contractClass.privateFunctions?.map(f => serializeToBuffer(f.selector, f.vkHash, f.isInternal)) ?? [],
    contractClass.publicFunctions.length,
    contractClass.publicFunctions?.map(f =>
      serializeToBuffer(f.selector, f.bytecode.length, f.bytecode, f.isInternal),
    ) ?? [],
    contractClass.packedBytecode.length,
    contractClass.packedBytecode,
    contractClass.privateFunctionsRoot,
  );
}

export function deserializeContractClassPublic(buffer: Buffer): Omit<ContractClassPublic, 'id'> {
  const reader = BufferReader.asReader(buffer);
  return {
    version: reader.readUInt8() as 1,
    artifactHash: reader.readObject(Fr),
    privateFunctions: reader.readVector({
      fromBuffer: reader => ({
        selector: reader.readObject(FunctionSelector),
        vkHash: reader.readObject(Fr),
        isInternal: reader.readBoolean(),
      }),
    }),
    publicFunctions: reader.readVector({
      fromBuffer: reader => ({
        selector: reader.readObject(FunctionSelector),
        bytecode: reader.readBuffer(),
        isInternal: reader.readBoolean(),
      }),
    }),
    packedBytecode: reader.readBuffer(),
    privateFunctionsRoot: reader.readObject(Fr),
  };
}
