import { AztecAddress, ContractFunctionDao } from '@aztec/circuits.js';
import {
  ContractArtifact,
  DebugFileMap,
  EventAbi,
  FunctionDebugMetadata,
  FunctionSelector,
  getFunctionDebugMetadata,
} from '@aztec/foundation/abi';
import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';
import { ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

/**
 * A contract Data Access Object (DAO).
 * Contains the contract's address, portal contract address, and an array of ContractFunctionDao objects.
 * Each ContractFunctionDao object includes FunctionAbi data and the function selector buffer.
 */
export class ContractDao implements ContractArtifact {
  /** An array of contract functions with additional selector property.  */
  public readonly functions: ContractFunctionDao[];

  constructor(private contractArtifact: ContractArtifact, public readonly instance: ContractInstanceWithAddress) {
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

  get fileMap(): DebugFileMap {
    return this.contractArtifact.fileMap;
  }

  getFunctionArtifact(selector: FunctionSelector): ContractFunctionDao | undefined {
    return this.functions.find(f => f.selector.equals(selector));
  }

  getFunctionArtifactByName(functionName: string): ContractFunctionDao | undefined {
    return this.functions.find(f => f.name === functionName);
  }

  getFunctionDebugMetadataByName(functionName: string): FunctionDebugMetadata | undefined {
    const fn = this.getFunctionArtifactByName(functionName);
    return fn && getFunctionDebugMetadata(this, fn);
  }

  toBuffer(): Buffer {
    // the contract artifact was originally emitted to a JSON file by Noir
    // should be safe to JSON.stringify it (i.e. it doesn't contain BigInts)
    const contractArtifactJson = JSON.stringify(this.contractArtifact);
    const buf = Buffer.concat([
      this.instance.address.toBuffer(),
      new SerializableContractInstance(this.instance).toBuffer(),
      prefixBufferWithLength(Buffer.from(contractArtifactJson, 'utf-8')),
    ]);

    return buf;
  }

  static fromBuffer(buf: Uint8Array | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const address = AztecAddress.fromBuffer(reader);
    const instance = SerializableContractInstance.fromBuffer(reader).withAddress(address);
    const contractArtifact = JSON.parse(reader.readString());
    return new ContractDao(contractArtifact, instance);
  }
}
