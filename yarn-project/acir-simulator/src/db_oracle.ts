import { AztecAddress, EthAddress, PrivateCircuitPublicInputs } from '@aztec/circuits.js';

export interface NoteLoadOracleInputs {
  note: Buffer;
  siblingPath: Buffer;
  leafIndex: number;
  root: Buffer;
}

export interface DBOracle {
  getSecretKey(contractAddress: AztecAddress, keyId: Buffer): Promise<Buffer>;
  getNotes(contractAddress: AztecAddress, storageSlot: Buffer): Promise<NoteLoadOracleInputs[]>;
  getBytecode(contractAddress: AztecAddress, functionSelector: string): Promise<Buffer>;
  getProvingKey(contractAddress: AztecAddress, functionSelector: string): Promise<Buffer>;
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;
}

export class PrivateCallStackItem {
  constructor(
    public readonly contractAddress: AztecAddress,
    public readonly functionSelector: number,
    public readonly publicInputs: PrivateCircuitPublicInputs,
  ) {}
}
