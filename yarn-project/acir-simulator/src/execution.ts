import {
  ACVMField,
  acvm,
  toACVMField,
  fromACVMField,
  ZERO_ACVM_FIELD,
  toAcvmCallPrivateStackItem,
  toAcvmNoteLoadOracleInputs,
  writeInputs,
} from './acvm/index.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { CallContext, OldTreeRoots, TxRequest, PrivateCallStackItem, FunctionData } from '@aztec/circuits.js';
import { DBOracle } from './db_oracle.js';
import { extractPublicInputs, frToAztecAddress, frToSelector } from './acvm/deserialize.js';
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
      this.contractAddress,
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
      privateFunctionCall: async ([acvmContractAddress, acvmFunctionSelector, ...acvmArgs]) => {
        const childExecutionResult = await this.privateFunctionCall(
          frToAztecAddress(fromACVMField(acvmContractAddress)),
          frToSelector(fromACVMField(acvmFunctionSelector)),
          acvmArgs.map(f => fromACVMField(f)),
          callContext,
        );

        nestedExecutionContexts.push(childExecutionResult);

        return toAcvmCallPrivateStackItem(childExecutionResult.callStackItem);
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
    const mapped = notes.flatMap(noteGetData =>
      toAcvmNoteLoadOracleInputs(noteGetData, this.oldRoots.privateDataTreeRoot),
    );
    return mapped;
  }

  private async getSecretKey(contractAddress: AztecAddress, address: ACVMField) {
    const key = await this.db.getSecretKey(contractAddress, frToAztecAddress(fromACVMField(address)));
    return [toACVMField(key)];
  }

  private async privateFunctionCall(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: Buffer,
    args: Fr[],
    callerContext: CallContext,
  ) {
    const abi = await this.db.getFunctionABI(targetContractAddress, targetFunctionSelector);
    const portalContractAddress = await this.db.getPortalContractAddress(targetContractAddress);
    const functionData = new FunctionData(targetFunctionSelector, true, false);
    const derivedCallContext = this.deriveCallContext(
      callerContext,
      targetContractAddress,
      portalContractAddress,
      false,
      false,
    );

    return this.runExternalFunction(abi, targetContractAddress, functionData, args, derivedCallContext);
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
