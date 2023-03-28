import { AztecAddress, CallContext, ContractDeploymentData } from '@aztec/circuits.js';
import { NoteLoadOracleInputs } from './db_oracle.js';

export interface ACIRCallback {
  getSecretKey(keyId: Buffer): Promise<Buffer>;
  getNotes(storageSlot: Buffer): Promise<NoteLoadOracleInputs[]>;
  getRandomField(): Promise<Buffer>;
  privateFunctionCall(
    contractAddress: AztecAddress,
    functionSelector: string,
    args: Array<Buffer>,
  ): Promise<Array<Buffer>>;
}

export interface ExecutionPreimages {
  newNotes: Buffer[];
  nullifiedNotes: Buffer[];
}

export interface ACIRExecutionResult {
  preimages: ExecutionPreimages;
  partialWitness: Buffer;
}

export type execute = (
  acir: Buffer,
  args: Array<Buffer>,
  callContext: CallContext,
  contractDeploymentData: ContractDeploymentData,
  oracle: ACIRCallback,
) => Promise<ACIRExecutionResult>;
