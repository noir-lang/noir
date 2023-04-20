import {
  ACVMField,
  acvm,
  toACVMField,
  fromACVMField,
  ZERO_ACVM_FIELD,
  toAcvmCallPrivateStackItem,
  toAcvmNoteLoadOracleInputs,
  writeInputs,
  createDummyNote,
} from './acvm/index.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import {
  CallContext,
  PrivateHistoricTreeRoots,
  TxRequest,
  PrivateCallStackItem,
  FunctionData,
  PRIVATE_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';
import { extractPublicInputs, frToAztecAddress, frToSelector } from './acvm/deserialize.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { createDebugLogger } from '@aztec/foundation/log';

export interface NewNoteData {
  preimage: Fr[];
  storageSlot: Fr;
  owner: { x: Fr; y: Fr };
}

export interface NewNullifierData {
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
    // Global to the tx
    private db: DBOracle,
    private request: TxRequest,
    private historicRoots: PrivateHistoricTreeRoots,
    // Concrete to this execution
    private abi: FunctionAbi,
    private contractAddress: AztecAddress,
    private functionData: FunctionData,
    private args: Fr[],
    private callContext: CallContext,

    private log = createDebugLogger('aztec:simulator:execution'),
  ) {}

  public async run(): Promise<ExecutionResult> {
    this.log(
      `Executing external function ${this.contractAddress.toShortString()}:${this.functionData.functionSelector.toString(
        'hex',
      )}`,
    );

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = writeInputs(this.args, this.callContext, this.request.txContext, this.historicRoots);
    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: ([address]: ACVMField[]) => {
        return this.getSecretKey(this.contractAddress, address);
      },
      getNotes2: async ([, storageSlot]: ACVMField[]) => {
        return await this.getNotes(this.contractAddress, storageSlot, 2);
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
      callPrivateFunction: async ([acvmContractAddress, acvmFunctionSelector, ...acvmArgs]) => {
        const childExecutionResult = await this.callPrivateFunction(
          frToAztecAddress(fromACVMField(acvmContractAddress)),
          frToSelector(fromACVMField(acvmFunctionSelector)),
          acvmArgs.map(f => fromACVMField(f)),
          this.callContext,
        );

        nestedExecutionContexts.push(childExecutionResult);

        return toAcvmCallPrivateStackItem(childExecutionResult.callStackItem);
      },
      storageRead: () => {
        return Promise.reject(new Error(`Storage access not available for private function execution`));
      },
      storageWrite: () => {
        return Promise.reject(new Error(`Storage access not available for private function execution`));
      },
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);

    return {
      acir,
      partialWitness,
      callStackItem,
      preimages: {
        newNotes: newNotePreimages,
        nullifiedNotes: newNullifiers,
      },
      vk: Buffer.from(this.abi.verificationKey!, 'hex'),
      nestedExecutions: nestedExecutionContexts,
    };
  }

  private async getNotes(contractAddress: AztecAddress, storageSlot: ACVMField, count: number) {
    const notes = await this.db.getNotes(contractAddress, fromACVMField(storageSlot), count);
    const dummyCount = Math.max(0, count - notes.length);
    const dummyNotes = Array.from({ length: dummyCount }, () => ({
      preimage: createDummyNote(),
      siblingPath: new Array(PRIVATE_DATA_TREE_HEIGHT).fill(Fr.ZERO),
      index: 0n,
    }));

    return notes
      .concat(dummyNotes)
      .flatMap(noteGetData => toAcvmNoteLoadOracleInputs(noteGetData, this.historicRoots.privateDataTreeRoot));
  }

  private async getSecretKey(contractAddress: AztecAddress, address: ACVMField) {
    // TODO remove this when we have brillig oracles that don't execute on false branches
    if (address === ZERO_ACVM_FIELD) {
      return [ZERO_ACVM_FIELD];
    }
    const key = await this.db.getSecretKey(contractAddress, frToAztecAddress(fromACVMField(address)));
    return [toACVMField(key)];
  }

  private async callPrivateFunction(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: Buffer,
    targetArgs: Fr[],
    callerContext: CallContext,
  ) {
    const targetAbi = await this.db.getFunctionABI(targetContractAddress, targetFunctionSelector);
    const targetPortalContractAddress = await this.db.getPortalContractAddress(targetContractAddress);
    const targetFunctionData = new FunctionData(targetFunctionSelector, true, false);
    const derivedCallContext = this.deriveCallContext(
      callerContext,
      targetContractAddress,
      targetPortalContractAddress,
      false,
      false,
    );

    const nestedExecution = new Execution(
      this.db,
      this.request,
      this.historicRoots,
      targetAbi,
      targetContractAddress,
      targetFunctionData,
      targetArgs,
      derivedCallContext,
    );

    return nestedExecution.run();
  }

  private deriveCallContext(
    parentContext: CallContext,
    targetContractAddress: AztecAddress,
    portalContractAddress: EthAddress,
    isDelegateCall = false,
    isStaticCall = false,
  ) {
    return new CallContext(
      parentContext.storageContractAddress,
      targetContractAddress,
      portalContractAddress,
      isDelegateCall,
      isStaticCall,
      false,
    );
  }
}
