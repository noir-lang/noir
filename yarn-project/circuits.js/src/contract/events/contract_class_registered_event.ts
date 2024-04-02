import { bufferFromFields } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';
import { type ContractClassPublic } from '@aztec/types/contracts';

import chunk from 'lodash.chunk';

import { REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE } from '../../constants.gen.js';
import { computeContractClassId, computePublicBytecodeCommitment } from '../contract_class_id.js';
import { unpackBytecode } from '../public_bytecode.js';

/** Event emitted from the ContractClassRegisterer. */
export class ContractClassRegisteredEvent {
  constructor(
    public readonly contractClassId: Fr,
    public readonly version: number,
    public readonly artifactHash: Fr,
    public readonly privateFunctionsRoot: Fr,
    public readonly packedPublicBytecode: Buffer,
  ) {}

  static isContractClassRegisteredEvent(log: Buffer) {
    return toBigIntBE(log.subarray(0, 32)) == REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE;
  }

  static fromLogs(logs: { contractAddress: AztecAddress; data: Buffer }[], registererContractAddress: AztecAddress) {
    return logs
      .filter(log => ContractClassRegisteredEvent.isContractClassRegisteredEvent(log.data))
      .filter(log => log.contractAddress.equals(registererContractAddress))
      .map(log => this.fromLogData(log.data));
  }

  static fromLogData(log: Buffer) {
    if (!this.isContractClassRegisteredEvent(log)) {
      throw new Error(
        `Log data for ContractClassRegisteredEvent is not prefixed with magic value 0x${REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE}`,
      );
    }
    const reader = new BufferReader(log.subarray(32));
    const contractClassId = reader.readObject(Fr);
    const version = reader.readObject(Fr).toNumber();
    const artifactHash = reader.readObject(Fr);
    const privateFunctionsRoot = reader.readObject(Fr);
    const packedPublicBytecode = bufferFromFields(
      chunk(reader.readToEnd(), Fr.SIZE_IN_BYTES).map(Buffer.from).map(Fr.fromBuffer),
    );

    return new ContractClassRegisteredEvent(
      contractClassId,
      version,
      artifactHash,
      privateFunctionsRoot,
      packedPublicBytecode,
    );
  }

  toContractClassPublic(): ContractClassPublic {
    const computedClassId = computeContractClassId({
      artifactHash: this.artifactHash,
      privateFunctionsRoot: this.privateFunctionsRoot,
      publicBytecodeCommitment: computePublicBytecodeCommitment(this.packedPublicBytecode),
    });

    if (!computedClassId.equals(this.contractClassId)) {
      throw new Error(
        `Invalid contract class id: computed ${computedClassId.toString()} but event broadcasted ${this.contractClassId.toString()}`,
      );
    }

    if (this.version !== 1) {
      throw new Error(`Unexpected contract class version ${this.version}`);
    }

    return {
      id: this.contractClassId,
      artifactHash: this.artifactHash,
      packedBytecode: this.packedPublicBytecode,
      privateFunctionsRoot: this.privateFunctionsRoot,
      publicFunctions: unpackBytecode(this.packedPublicBytecode),
      version: this.version,
      privateFunctions: [],
      unconstrainedFunctions: [],
    };
  }
}
