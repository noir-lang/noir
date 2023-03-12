import { hexToBuffer } from '../../hex_string/index.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

export class ContractFunctionEntry extends ContractEntry {
  public readonly signature: string;

  constructor(entry: ContractEntryDefinition) {
    entry.inputs = entry.inputs || [];
    super(entry);
    this.signature =
      entry.type === 'constructor'
        ? 'constructor'
        : abiCoder.encodeFunctionSignature(abiCoder.abiMethodToString(entry));
  }

  public get constant() {
    return this.entry.stateMutability === 'view' || this.entry.stateMutability === 'pure' || this.entry.constant;
  }

  public get payable() {
    return this.entry.stateMutability === 'payable' || this.entry.payable;
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
    return Buffer.concat([hexToBuffer(this.signature), this.encodeParameters(args)]);
  }

  public encodeParameters(args: any[]) {
    return abiCoder.encodeParameters(this.entry.inputs, args);
  }

  public decodeParameters(bytes: Buffer) {
    return abiCoder.decodeParameters(this.entry.inputs, bytes);
  }
}
