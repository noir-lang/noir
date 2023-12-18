import { CompleteAddress, ContractFunctionDao } from '@aztec/circuits.js';
import { ContractArtifact, DebugMetadata, EventAbi, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';
import { prefixBufferWithLength } from '@aztec/foundation/serialize';

import { BufferReader, EncodedContractFunction } from './contract_data.js';

/**
 * A contract Data Access Object (DAO).
 * Contains the contract's address, portal contract address, and an array of ContractFunctionDao objects.
 * Each ContractFunctionDao object includes FunctionAbi data and the function selector buffer.
 */
export class ContractDao implements ContractArtifact {
  /** An array of contract functions with additional selector property.  */
  public readonly functions: ContractFunctionDao[];
  constructor(
    private contractArtifact: ContractArtifact,
    /** The complete address representing the contract on L2.  */
    public readonly completeAddress: CompleteAddress,
    /** The Ethereum address of the L1 contract serving as a bridge for cross-layer interactions.  */
    public readonly portalContract: EthAddress,
  ) {
    this.functions = contractArtifact.functions.map(f => ({
      ...f,
      selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters),
    }));
  }

  get aztecNrVersion() {
    return this.contractArtifact.aztecNrVersion;
  }

  get name(): string {
    return this.contractArtifact.name;
  }

  get events(): EventAbi[] {
    return this.contractArtifact.events;
  }

  get debug(): DebugMetadata | undefined {
    return this.contractArtifact.debug;
  }

  toBuffer(): Buffer {
    // the contract artifact was originally emitted to a JSON file by Noir
    // should be safe to JSON.stringify it (i.e. it doesn't contain BigInts)
    const contractArtifactJson = JSON.stringify(this.contractArtifact);
    const buf = Buffer.concat([
      this.completeAddress.toBuffer(),
      this.portalContract.toBuffer20(),
      prefixBufferWithLength(Buffer.from(contractArtifactJson, 'utf-8')),
    ]);

    return buf;
  }

  static fromBuffer(buf: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const completeAddress = CompleteAddress.fromBuffer(reader);
    const portalContract = new EthAddress(reader.readBytes(EthAddress.SIZE_IN_BYTES));
    const contractArtifact = JSON.parse(reader.readString());
    return new ContractDao(contractArtifact, completeAddress, portalContract);
  }
}

/**
 * Return public functions from the newly deployed contract to be injected into the tx object.
 * @param newContract - The new contract
 * @returns List of EncodedContractFunction.
 */
export function getNewContractPublicFunctions(newContract: ContractDao) {
  return newContract.functions
    .filter(c => c.functionType === FunctionType.OPEN)
    .map(
      fn =>
        new EncodedContractFunction(
          FunctionSelector.fromNameAndParameters(fn.name, fn.parameters),
          fn.isInternal ?? false,
          Buffer.from(fn.bytecode, 'base64'),
        ),
    );
}
