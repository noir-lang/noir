import { ACVMField, acvm, toACVMField, fromACVMField, ZERO_ACVM_FIELD } from './acvm.js';
import {
  AztecAddress,
  CallContext,
  EthAddress,
  Fr,
  FunctionData,
  OldTreeRoots,
  PrivateCallStackItem,
  TxRequest,
} from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';
import { writeInputs, extractPublicInputs, frToAztecAddress } from './witness_io.js';
import { FunctionAbi } from '@aztec/noir-contracts';

export interface ExecutionPreimages {
  newNotes: Array<{ preimage: Fr[]; storageSlot: Fr }>;
  nullifiedNotes: Fr[];
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
      this.request.functionData.functionSelector,
      this.request.args,
      callContext,
    );
  }

  // Separate function so we can recurse in the future
  private async runExternalFunction(
    abi: FunctionAbi,
    contractAddress: AztecAddress,
    functionSelector: Buffer,
    args: Fr[],
    callContext: CallContext,
  ): Promise<ExecutionResult> {
    const acir = Buffer.from(abi.bytecode, 'hex');
    const initialWitness = writeInputs(args, callContext, this.request.txContext, this.oldRoots);
    const newNotePreimages: Array<{ preimage: Fr[]; storageSlot: Fr }> = [];
    const newNullifiers: Fr[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: ([address]: ACVMField[]) => {
        return this.getSecretKey(contractAddress, address);
      },
      getNotes2: async ([storageSlot]: ACVMField[]) => {
        return await this.getNotes(contractAddress, storageSlot, 2);
      },
      getRandomField: () => Promise.resolve([toACVMField(Fr.random())]),
      notifyCreatedNote: (params: ACVMField[]) => {
        const [storageSlot, ...acvmPreimage] = params;
        const preimage = acvmPreimage.map(f => fromACVMField(f));
        newNotePreimages.push({
          preimage,
          storageSlot: fromACVMField(storageSlot),
        });
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
      notifyNullifiedNote: ([nullifier]: ACVMField[]) => {
        newNullifiers.push(fromACVMField(nullifier));
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    const callStackItem = new PrivateCallStackItem(contractAddress, new FunctionData(functionSelector), publicInputs);

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
    const notes = await this.db.getNotes(contractAddress, fromACVMField(storageSlot));
    const mapped = notes
      .slice(0, count)
      .flatMap(note => [
        toACVMField(note.index),
        ...note.note.map(f => toACVMField(f)),
        toACVMField(this.oldRoots.privateDataTreeRoot),
        ...note.siblingPath.map(f => toACVMField(f)),
      ]);
    return mapped;
  }

  private async getSecretKey(contractAddress: AztecAddress, address: ACVMField) {
    const key = await this.db.getSecretKey(contractAddress, frToAztecAddress(fromACVMField(address)));
    return [toACVMField(key)];
  }
}
