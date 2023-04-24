import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { FunctionAbi } from '@aztec/noir-contracts';

export interface NoteLoadOracleInputs {
  preimage: Fr[];
  siblingPath: Fr[];
  index: bigint;
}

export interface DBOracle {
  getSecretKey(contractAddress: AztecAddress, address: AztecAddress): Promise<Buffer>;
  getNotes(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    limit: number,
    offset: number,
  ): Promise<{ count: number; notes: NoteLoadOracleInputs[] }>;
  getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi>;
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;
}
