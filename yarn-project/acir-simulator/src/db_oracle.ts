import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';

export interface NoteLoadOracleInputs {
  preimage: Fr[];
  siblingPath: Fr[];
  index: number;
}

export interface DBOracle {
  getSecretKey(contractAddress: AztecAddress, address: AztecAddress): Promise<Buffer>;
  getNotes(contractAddress: AztecAddress, storageSlot: Fr, count: number): Promise<NoteLoadOracleInputs[]>;
  getBytecode(contractAddress: AztecAddress, functionSelector: Buffer): Promise<Buffer>;
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;
}
