export type AbiDataTypes = 'bool' | 'string' | 'address' | 'function' | 'uint' | 'int' | 'bytes' | string;

export type AbiInput = {
  components?: any;
  name: string;
  type: AbiDataTypes;
  indexed?: boolean;
  internalType?: string;
};

export type AbiOutput = {
  components?: any;
  name: string;
  type: AbiDataTypes;
  internalType?: string;
};

export interface ContractEntryDefinition {
  constant?: boolean;
  payable?: boolean;
  anonymous?: boolean;
  inputs?: AbiInput[];
  name?: string;
  outputs?: AbiOutput[];
  type: 'function' | 'constructor' | 'event' | 'fallback' | 'error' | 'receive';
  stateMutability?: 'pure' | 'view' | 'payable' | 'nonpayable';
  signature?: string;
  gas?: number;
}

export type ContractAbiDefinition = ContractEntryDefinition[];
