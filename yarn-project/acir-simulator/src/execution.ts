import { ACVMField, acvm, toACVMField, fromACVMField, ZERO_ACVM_FIELD } from './acvm/index.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { CallContext, OldTreeRoots, TxRequest, PrivateCallStackItem, FunctionData } from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';
import { writeInputs, extractPublicInputs, frToAztecAddress } from './acvm/witness_io.js';
import { FunctionAbi } from '@aztec/noir-contracts';

interface NewNoteData {
  preimage: Fr[];
  storageSlot: Fr;
  owner: { x: Fr; y: Fr };
}

interface NewNullifierData {
  preimage: Fr[];
  storageSlot: Fr;
  nullifier: Fr;
}

export interface ExecutionPreimages {
  newNotes: NewNoteData[];
  nullifiedNotes: NewNullifierData[];
}

export interface ExecutionResult {
  // Needed for prover
  acir: Buffer;
  vk: Buffer;
  partialWitness: Map<number, ACVMField>;
  // Needed for the verifier (kernel)
  callStackItem: PrivateCallStackItem;
  // Needed for the user
  preimages: ExecutionPreimages;
  // Nested executions
  nestedExecutions: this[];
}

export class Execution {
  constructor(
    private db: DBOracle,
    private request: TxRequest,
    private entryPointABI: FunctionAbi,
    private contractAddress: AztecAddress,
    private portalContractAddress: EthAddress,
    private oldRoots: OldTreeRoots,
  ) {}

  public run(): Promise<ExecutionResult> {
    const callContext = new CallContext(
      this.request.from,
      this.request.to,
      this.portalContractAddress,
      false,
      false,
      this.request.functionData.isConstructor,
    );

    return this.runExternalFunction(
      this.entryPointABI,
      this.contractAddress,
      this.request.functionData,
      this.request.args,
      callContext,
    );
  }

  // Separate function so we can recurse in the future
  private async runExternalFunction(
    abi: FunctionAbi,
    contractAddress: AztecAddress,
    functionData: FunctionData,
    args: Fr[],
    callContext: CallContext,
  ): Promise<ExecutionResult> {
    const acir = Buffer.from(abi.bytecode, 'hex');
    const initialWitness = writeInputs(args, callContext, this.request.txContext, this.oldRoots);
    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: ([address]: ACVMField[]) => {
        return this.getSecretKey(contractAddress, address);
      },
      getNotes2: async ([, storageSlot]: ACVMField[]) => {
        return await this.getNotes(contractAddress, storageSlot, 2);
      },
      getRandomField: () => Promise.resolve([toACVMField(Fr.random())]),
      notifyCreatedNote: ([storageSlot, ownerX, ownerY, ...acvmPreimage]: ACVMField[]) => {
        newNotePreimages.push({
          preimage: acvmPreimage.map(f => fromACVMField(f)),
          storageSlot: fromACVMField(storageSlot),
          owner: {
            x: fromACVMField(ownerX),
            y: fromACVMField(ownerY),
          },
        });
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
      notifyNullifiedNote: ([slot, nullifier, ...acvmPreimage]: ACVMField[]) => {
        newNullifiers.push({
          preimage: acvmPreimage.map(f => fromACVMField(f)),
          storageSlot: fromACVMField(slot),
          nullifier: fromACVMField(nullifier),
        });
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    const callStackItem = new PrivateCallStackItem(contractAddress, functionData, publicInputs);

    return {
      acir,
      partialWitness,
      callStackItem,
      preimages: {
        newNotes: newNotePreimages,
        nullifiedNotes: newNullifiers,
      },
      vk: Buffer.from(abi.verificationKey!, 'hex'),
      nestedExecutions: nestedExecutionContexts,
    };
  }

  private async getNotes(contractAddress: AztecAddress, storageSlot: ACVMField, count: number) {
    const notes = await this.db.getNotes(contractAddress, fromACVMField(storageSlot), count);
    const mapped = notes.flatMap(noteGetData => [
      ...noteGetData.preimage.map(f => toACVMField(f)),
      toACVMField(noteGetData.index),
      ...noteGetData.siblingPath.map(f => toACVMField(f)),
      toACVMField(this.oldRoots.privateDataTreeRoot),
    ]);
    return mapped;
  }

  private async getSecretKey(contractAddress: AztecAddress, address: ACVMField) {
    const key = await this.db.getSecretKey(contractAddress, frToAztecAddress(fromACVMField(address)));
    return [toACVMField(key)];
  }
}
