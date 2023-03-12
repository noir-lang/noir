import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';

export class ContractEntry {
  constructor(protected entry: ContractEntryDefinition) {}

  public get name() {
    return this.entry.name;
  }

  public get anonymous() {
    return this.entry.anonymous || false;
  }

  public asString() {
    return abiCoder.abiMethodToString(this.entry);
  }
}
