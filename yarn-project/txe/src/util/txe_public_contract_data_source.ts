import { type AztecAddress, Fr, type FunctionSelector, unpackBytecode } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { PrivateFunctionsTree } from '@aztec/pxe';
import {
  type ContractClassPublic,
  type ContractDataSource,
  type ContractInstanceWithAddress,
  type PublicFunction,
} from '@aztec/types/contracts';

import { type TXE } from '../oracle/txe_oracle.js';

export class TXEPublicContractDataSource implements ContractDataSource {
  constructor(private txeOracle: TXE) {}

  async getPublicFunction(address: AztecAddress, selector: FunctionSelector): Promise<PublicFunction | undefined> {
    const bytecode = await this.txeOracle.getContractDataOracle().getBytecode(address, selector);
    if (!bytecode) {
      return undefined;
    }
    return { bytecode, selector };
  }

  getBlockNumber(): Promise<number> {
    return this.txeOracle.getBlockNumber();
  }

  async getContractClass(id: Fr): Promise<ContractClassPublic | undefined> {
    const contractClass = await this.txeOracle.getContractDataOracle().getContractClass(id);
    const artifact = await this.txeOracle.getContractDataOracle().getContractArtifact(id);
    const tree = new PrivateFunctionsTree(artifact);
    const privateFunctionsRoot = tree.getFunctionTreeRoot();

    return {
      id,
      artifactHash: contractClass!.artifactHash,
      packedBytecode: contractClass!.packedBytecode,
      publicFunctions: unpackBytecode(contractClass!.packedBytecode),
      privateFunctionsRoot: new Fr(privateFunctionsRoot!.root),
      version: contractClass!.version,
      privateFunctions: [],
      unconstrainedFunctions: [],
    };
  }

  async getContract(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    const instance = await this.txeOracle.getContractDataOracle().getContractInstance(address);
    return { ...instance, address };
  }

  getContractClassIds(): Promise<Fr[]> {
    throw new Error('Method not implemented.');
  }

  async getContractArtifact(address: AztecAddress): Promise<ContractArtifact | undefined> {
    const instance = await this.txeOracle.getContractDataOracle().getContractInstance(address);
    return this.txeOracle.getContractDataOracle().getContractArtifact(instance.contractClassId);
  }

  addContractArtifact(address: AztecAddress, contract: ContractArtifact): Promise<void> {
    return this.txeOracle.addContractArtifact(contract);
  }
}
