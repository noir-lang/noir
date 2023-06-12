import {
  ARGS_LENGTH,
  CallContext,
  CircuitsWasm,
  ContractDeploymentData,
  FunctionData,
  PUBLIC_CALL_STACK_LENGTH,
  PrivateCallStackItem,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { computeCallStackItemHash, computeVarArgsHash } from '@aztec/circuits.js/abis';
import { FunctionAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { decodeReturnValues } from '../abi_coder/decoder.js';
import { extractPublicInputs, frToAztecAddress, frToSelector } from '../acvm/deserialize.js';
import {
  ACVMField,
  ZERO_ACVM_FIELD,
  acvm,
  convertACVMFieldToBuffer,
  fromACVMField,
  toACVMField,
  toACVMWitness,
  toAcvmCallPrivateStackItem,
  toAcvmEnqueuePublicFunctionResult,
} from '../acvm/index.js';
import { sizeOfType } from '../index.js';
import { fieldsToFormattedStr } from './debug.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { Tuple, assertLength } from '@aztec/foundation/serialize';
import { NoirLogs, NotePreimage, TxAuxData } from '@aztec/types';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';

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
  /**
   * Encrypted logs emitted during execution of this function call.
   * Note: These are preimages to `encryptedLogsHash`.
   */
  encryptedLogs: NoirLogs;
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
    const selector = this.functionData.functionSelectorBuffer.toString('hex');
    this.log(`Executing external function ${this.contractAddress.toShortString()}:${selector}`);

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = this.writeInputs();

    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];
    const enqueuedPublicFunctionCalls: PublicCallRequest[] = [];
    const encryptedLogs = new NoirLogs([]);

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
        const contractAddress = fromACVMField(acvmContractAddress);
        const functionSelector = fromACVMField(acvmFunctionSelector);
        this.log(
          `Calling private function ${contractAddress.toShortString()}:${functionSelector} from ${this.callContext.storageContractAddress.toShortString()}`,
        );

        const childExecutionResult = await this.callPrivateFunction(
          frToAztecAddress(contractAddress),
          frToSelector(functionSelector),
          acvmArgs.map(f => fromACVMField(f)),
          this.callContext,
        );

        nestedExecutionContexts.push(childExecutionResult);

        return toAcvmCallPrivateStackItem(childExecutionResult.callStackItem);
      },
      getL1ToL2Message: ([msgKey]: ACVMField[]) => this.context.getL1ToL2Message(fromACVMField(msgKey)),

      debugLog: (fields: ACVMField[]) => {
        this.log(fieldsToFormattedStr(fields));
        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
      enqueuePublicFunctionCall: async ([acvmContractAddress, acvmFunctionSelector, ...acvmArgs]) => {
        const enqueuedRequest = await this.enqueuePublicFunctionCall(
          frToAztecAddress(fromACVMField(acvmContractAddress)),
          frToSelector(fromACVMField(acvmFunctionSelector)),
          assertLength(
            acvmArgs.map(f => fromACVMField(f)),
            ARGS_LENGTH,
          ),
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
      emitEncryptedLog: async ([
        acvmContractAddress,
        acvmStorageSlot,
        ownerX,
        ownerY,
        ...acvmPreimage
      ]: ACVMField[]) => {
        const contractAddress = AztecAddress.fromBuffer(convertACVMFieldToBuffer(acvmContractAddress));
        const storageSlot = fromACVMField(acvmStorageSlot);
        const preimage = acvmPreimage.map(f => fromACVMField(f));

        const notePreimage = new NotePreimage(preimage);
        const txAuxData = new TxAuxData(notePreimage, contractAddress, storageSlot);
        const ownerPublicKey = new Point(
          Buffer.concat([convertACVMFieldToBuffer(ownerX), convertACVMFieldToBuffer(ownerY)]),
        );

        const encryptedNotePreimage = txAuxData.toEncryptedBuffer(ownerPublicKey, await Grumpkin.new());

        encryptedLogs.dataChunks.push(encryptedNotePreimage);

        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    // TODO(#499): Noir fails to compute the args hash, so we patch those values here.
    const wasm = await CircuitsWasm.get();
    publicInputs.argsHash = await computeVarArgsHash(wasm, this.args);

    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);
    const returnValues = decodeReturnValues(this.abi, publicInputs.returnValues);

    // TODO(#499): Noir fails to compute the enqueued calls preimages properly, since it cannot use pedersen generators, so we patch those values here.
    const publicCallStackItems = await Promise.all(enqueuedPublicFunctionCalls.map(c => c.toPublicCallStackItem()));
    const publicStack = await Promise.all(publicCallStackItems.map(c => computeCallStackItemHash(wasm, c)));
    callStackItem.publicInputs.publicCallStack = padArrayEnd(publicStack, Fr.ZERO, PUBLIC_CALL_STACK_LENGTH);

    this.log(`Returning from call to ${this.contractAddress.toShortString()}:${selector}`);

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
      encryptedLogs,
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
    const contractDeploymentData = this.context.txContext.contractDeploymentData ?? ContractDeploymentData.empty();

    // NOTE: PSA to anyone updating this code: within the structs, the members must be in alphabetical order, this
    // is a current quirk in noir struct encoding, feel free to remove this note when this changes
    const fields = [
      this.callContext.isContractDeployment,
      this.callContext.isDelegateCall,
      this.callContext.isStaticCall,
      this.callContext.msgSender,
      this.callContext.portalContractAddress,
      this.callContext.storageContractAddress,

      contractDeploymentData.constructorVkHash,
      contractDeploymentData.contractAddressSalt,
      contractDeploymentData.functionTreeRoot,
      contractDeploymentData.portalContractAddress,

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
    targetArgs: Tuple<Fr, typeof ARGS_LENGTH>,
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
