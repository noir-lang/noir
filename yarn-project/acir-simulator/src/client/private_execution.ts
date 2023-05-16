import {
  CallContext,
  CircuitsWasm,
  FunctionData,
  PUBLIC_CALL_STACK_LENGTH,
  PrivateCallStackItem,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { computeCallStackItemHash } from '@aztec/circuits.js/abis';
import { FunctionAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { decodeReturnValues } from '../abi_coder/decoder.js';
import { extractPublicInputs, frToAztecAddress, frToSelector } from '../acvm/deserialize.js';
import {
  ACVMField,
  ZERO_ACVM_FIELD,
  acvm,
  fromACVMField,
  toACVMField,
  toACVMWitness,
  toAcvmCallPrivateStackItem,
  toAcvmEnqueuePublicFunctionResult,
} from '../acvm/index.js';
import { sizeOfType } from '../index.js';
import { ClientTxExecutionContext } from './client_execution_context.js';

/**
 * The contents of a new note.
 */
export interface NewNoteData {
  /** The preimage of the note. */
  preimage: Fr[];
  /** The storage slot of the note. */
  storageSlot: Fr;
  /** The note owner. */
  owner: {
    /** The x coordinate. */
    x: Fr;
    /** The y coordinate. */
    y: Fr;
  };
}

/**
 * The contents of a nullified commitment.
 */
export interface NewNullifierData {
  /** The preimage of the nullified commitment. */
  preimage: Fr[];
  /** The storage slot of the nullified commitment. */
  storageSlot: Fr;
  /** The nullifier. */
  nullifier: Fr;
}

/**
 * The preimages of the executed function.
 */
export interface ExecutionPreimages {
  /** The preimages of the new notes. */
  newNotes: NewNoteData[];
  /** The preimages of the nullified commitments. */
  nullifiedNotes: NewNullifierData[];
}

/**
 * The result of executing a private function.
 */
export interface ExecutionResult {
  // Needed for prover
  /** The ACIR bytecode. */
  acir: Buffer;
  /** The verification key. */
  vk: Buffer;
  /** The partial witness. */
  partialWitness: Map<number, ACVMField>;
  // Needed for the verifier (kernel)
  /** The call stack item. */
  callStackItem: PrivateCallStackItem;
  // Needed for the user
  /** The preimages of the executed function. */
  preimages: ExecutionPreimages;
  /** The decoded return values of the executed function. */
  returnValues: any[];
  /** The nested executions. */
  nestedExecutions: this[];
  /** Enqueued public function execution requests to be picked up by the sequencer. */
  enqueuedPublicFunctionCalls: PublicCallRequest[];
}

const notAvailable = () => {
  return Promise.reject(new Error(`Not available for private function execution`));
};

/**
 * The private function execution class.
 */
export class PrivateFunctionExecution {
  constructor(
    private context: ClientTxExecutionContext,
    private abi: FunctionAbi,
    private contractAddress: AztecAddress,
    private functionData: FunctionData,
    private args: Fr[],
    private callContext: CallContext,

    private log = createDebugLogger('aztec:simulator:secret_execution'),
  ) {}

  /**
   * Executes the function.
   * @returns The execution result.
   */
  public async run(): Promise<ExecutionResult> {
    const selector = this.functionData.functionSelector.toString('hex');
    this.log(`Executing external function ${this.contractAddress.toShortString()}:${selector}`);

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = this.writeInputs();

    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];
    const enqueuedPublicFunctionCalls: PublicCallRequest[] = [];

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: async ([address]: ACVMField[]) => [
        toACVMField(await this.context.db.getSecretKey(this.contractAddress, frToAztecAddress(fromACVMField(address)))),
      ],
      getNotes2: ([storageSlot]: ACVMField[]) => this.context.getNotes(this.contractAddress, storageSlot, 2),
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
      enqueuePublicFunctionCall: async ([acvmContractAddress, acvmFunctionSelector, ...acvmArgs]) => {
        const enqueuedRequest = await this.enqueuePublicFunctionCall(
          frToAztecAddress(fromACVMField(acvmContractAddress)),
          frToSelector(fromACVMField(acvmFunctionSelector)),
          acvmArgs.map(f => fromACVMField(f)),
          this.callContext,
        );

        this.log(`Enqueued call to public function ${acvmContractAddress}:${acvmFunctionSelector}`);
        enqueuedPublicFunctionCalls.push(enqueuedRequest);
        return toAcvmEnqueuePublicFunctionResult(enqueuedRequest);
      },
      viewNotesPage: notAvailable,
      storageRead: notAvailable,
      storageWrite: notAvailable,
      callPublicFunction: notAvailable,
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);
    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);

    const returnValues = decodeReturnValues(this.abi, publicInputs.returnValues);

    // TODO: Noir fails to compute the enqueued calls preimages properly, since it cannot use pedersen
    // generators, so we patch those values here. See https://github.com/AztecProtocol/aztec-packages/issues/499.
    const wasm = await CircuitsWasm.get();
    const publicStack = enqueuedPublicFunctionCalls.map(c => computeCallStackItemHash(wasm, c.toPublicCallStackItem()));
    callStackItem.publicInputs.publicCallStack = padArrayEnd(publicStack, Fr.ZERO, PUBLIC_CALL_STACK_LENGTH);

    return {
      acir,
      partialWitness,
      callStackItem,
      returnValues,
      preimages: {
        newNotes: newNotePreimages,
        nullifiedNotes: newNullifiers,
      },
      vk: Buffer.from(this.abi.verificationKey!, 'hex'),
      nestedExecutions: nestedExecutionContexts,
      enqueuedPublicFunctionCalls,
    };
  }

  // We still need this function until we can get user-defined ordering of structs for fn arguments
  // TODO When that is sorted out on noir side, we can use instead the utilities in serialize.ts
  /**
   * Writes the function inputs to the initial witness.
   * @returns The initial witness.
   */
  private writeInputs() {
    const argsSize = this.abi.parameters.reduce((acc, param) => acc + sizeOfType(param.type), 0);

    // NOTE: PSA to anyone updating this code: within the structs, the members must be in alphabetical order, this
    // is a current quirk in noir struct encoding, feel free to remove this note when this changes
    const fields = [
      this.callContext.isContractDeployment,
      this.callContext.isDelegateCall,
      this.callContext.isStaticCall,
      this.callContext.msgSender,
      this.callContext.portalContractAddress,
      this.callContext.storageContractAddress,

      this.context.request.txContext.contractDeploymentData.constructorVkHash,
      this.context.request.txContext.contractDeploymentData.contractAddressSalt,
      this.context.request.txContext.contractDeploymentData.functionTreeRoot,
      this.context.request.txContext.contractDeploymentData.portalContractAddress,

      this.context.historicRoots.contractTreeRoot,
      this.context.historicRoots.l1ToL2MessagesTreeRoot,
      this.context.historicRoots.nullifierTreeRoot,
      this.context.historicRoots.privateDataTreeRoot,

      ...this.args.slice(0, argsSize),
    ];

    return toACVMWitness(1, fields);
  }

  /**
   * Calls a private function as a nested execution.
   * @param targetContractAddress - The address of the contract to call.
   * @param targetFunctionSelector - The function selector of the function to call.
   * @param targetArgs - The arguments to pass to the function.
   * @param callerContext - The call context of the caller.
   * @returns The execution result.
   */
  private async callPrivateFunction(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: Buffer,
    targetArgs: Fr[],
    callerContext: CallContext,
  ) {
    const targetAbi = await this.context.db.getFunctionABI(targetContractAddress, targetFunctionSelector);
    const targetFunctionData = new FunctionData(targetFunctionSelector, true, false);
    const derivedCallContext = await this.deriveCallContext(callerContext, targetContractAddress, false, false);

    const nestedExecution = new PrivateFunctionExecution(
      this.context,
      targetAbi,
      targetContractAddress,
      targetFunctionData,
      targetArgs,
      derivedCallContext,
    );

    return nestedExecution.run();
  }

  /**
   * Creates a PublicCallStackItem object representing the request to call a public function. No function
   * is actually called, since that must happen on the sequencer side. All the fields related to the result
   * of the execution are empty.
   * @param targetContractAddress - The address of the contract to call.
   * @param targetFunctionSelector - The function selector of the function to call.
   * @param targetArgs - The arguments to pass to the function.
   * @param callerContext - The call context of the caller.
   * @returns The public call stack item with the request information.
   */
  private async enqueuePublicFunctionCall(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: Buffer,
    targetArgs: Fr[],
    callerContext: CallContext,
  ): Promise<PublicCallRequest> {
    const derivedCallContext = await this.deriveCallContext(callerContext, targetContractAddress, false, false);
    return PublicCallRequest.from({
      args: targetArgs,
      callContext: derivedCallContext,
      functionData: new FunctionData(targetFunctionSelector, false, false),
      contractAddress: targetContractAddress,
    });
  }

  /**
   * Derives the call context for a nested execution.
   * @param parentContext - The parent call context.
   * @param targetContractAddress - The address of the contract being called.
   * @param isDelegateCall - Whether the call is a delegate call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The derived call context.
   */
  private async deriveCallContext(
    parentContext: CallContext,
    targetContractAddress: AztecAddress,
    isDelegateCall = false,
    isStaticCall = false,
  ) {
    const portalContractAddress = await this.context.db.getPortalContractAddress(targetContractAddress);
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
