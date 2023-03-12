import { hexToBuffer } from '../../hex_string/index.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

export class ContractErrorEntry extends ContractEntry {
  public readonly signature: Buffer;

  constructor(entry: ContractEntryDefinition) {
    entry.inputs = entry.inputs || [];
    super(entry);
    this.signature = hexToBuffer(abiCoder.encodeFunctionSignature(abiCoder.abiMethodToString(entry)));
  }

  public numArgs() {
    return this.entry.inputs ? this.entry.inputs.length : 0;
  }

  public decodeReturnValue(returnValue: Buffer) {
    if (!returnValue.length) {
      return null;
    }

    const result = abiCoder.decodeParameters(this.entry.outputs, returnValue);

    if (result.__length__ === 1) {
      return result[0];
    } else {
      delete result.__length__;
      return result;
    }
  }

  public encodeABI(args: any[]) {
    return Buffer.concat([this.signature, this.encodeParameters(args)]);
  }

  public encodeParameters(args: any[]) {
    return abiCoder.encodeParameters(this.entry.inputs, args);
  }

  public decodeParameters(bytes: Buffer) {
    return abiCoder.decodeParameters(this.entry.inputs, bytes);
  }
}
