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
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { FunctionAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { Tuple, assertLength, to2Fields } from '@aztec/foundation/serialize';
import { FunctionL2Logs, NotePreimage, NoteSpendingInfo } from '@aztec/types';
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
import { ExecutionResult, NewNoteData, NewNullifierData, sizeOfType } from '../index.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { fieldsToFormattedStr } from './debug.js';

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
    this.log(`Executing external function ${this.contractAddress.toString()}:${selector}`);

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = this.writeInputs();

    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];
    const enqueuedPublicFunctionCalls: PublicCallRequest[] = [];
    const readRequestCommitmentIndices: bigint[] = [];
    const encryptedLogs = new FunctionL2Logs([]);

    const { partialWitness } = await acvm(acir, initialWitness, {
      getSecretKey: async ([ownerX, ownerY]: ACVMField[]) => [
        toACVMField(
          await this.context.db.getSecretKey(
            this.contractAddress,
            Point.fromCoordinates(fromACVMField(ownerX), fromACVMField(ownerY)),
          ),
        ),
      ],
      getNotes2: async ([storageSlot]: ACVMField[]) => {
        const { preimages, indices } = await this.context.getNotes(this.contractAddress, storageSlot, 2);
        // TODO(dbanks12): https://github.com/AztecProtocol/aztec-packages/issues/779
        // if preimages length is > rrcIndices length, we are either relying on
        // the app circuit to remove fake preimages, or on the kernel to handle
        // the length diff.
        const filteredIndices = indices.filter(index => index != BigInt(-1));
        readRequestCommitmentIndices.push(...filteredIndices);
        return preimages;
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
        const contractAddress = fromACVMField(acvmContractAddress);
        const functionSelector = fromACVMField(acvmFunctionSelector);
        this.log(
          `Calling private function ${contractAddress.toString()}:${functionSelector} from ${this.callContext.storageContractAddress.toString()}`,
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
      getL1ToL2Message: ([msgKey]: ACVMField[]) => {
        return this.context.getL1ToL2Message(fromACVMField(msgKey));
      },
      getCommitment: async ([commitment]: ACVMField[]) => {
        const commitmentData = await this.context.getCommitment(this.contractAddress, fromACVMField(commitment));
        readRequestCommitmentIndices.push(commitmentData.index);
        return commitmentData.acvmData;
      },
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
      createCommitment: notAvailable,
      createL2ToL1Message: notAvailable,
      callPublicFunction: notAvailable,
      emitUnencryptedLog: notAvailable,
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
        const noteSpendingInfo = new NoteSpendingInfo(notePreimage, contractAddress, storageSlot);
        const ownerPublicKey = new Point(
          Buffer.concat([convertACVMFieldToBuffer(ownerX), convertACVMFieldToBuffer(ownerY)]),
        );

        const encryptedNotePreimage = noteSpendingInfo.toEncryptedBuffer(ownerPublicKey, await Grumpkin.new());

        encryptedLogs.logs.push(encryptedNotePreimage);

        return Promise.resolve([ZERO_ACVM_FIELD]);
      },
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    // TODO(#499): Noir fails to compute the args hash, so we patch those values here.
    const wasm = await CircuitsWasm.get();
    publicInputs.argsHash = await computeVarArgsHash(wasm, this.args);

    // TODO(#1347): Noir fails with too many unknowns error when public inputs struct contains too many members.
    publicInputs.encryptedLogsHash = to2Fields(encryptedLogs.hash());
    publicInputs.encryptedLogPreimagesLength = new Fr(encryptedLogs.getSerializedLength());

    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);
    const returnValues = decodeReturnValues(this.abi, publicInputs.returnValues);

    // TODO(#499): Noir fails to compute the enqueued calls preimages properly, since it cannot use pedersen generators, so we patch those values here.
    const publicCallStackItems = await Promise.all(enqueuedPublicFunctionCalls.map(c => c.toPublicCallStackItem()));
    const publicStack = await Promise.all(publicCallStackItems.map(c => computeCallStackItemHash(wasm, c)));
    callStackItem.publicInputs.publicCallStack = padArrayEnd(publicStack, Fr.ZERO, PUBLIC_CALL_STACK_LENGTH);

    // TODO: This should be set manually by the circuit
    publicInputs.contractDeploymentData.deployerPublicKey =
      this.context.txContext.contractDeploymentData.deployerPublicKey;

    this.log(`Returning from call to ${this.contractAddress.toString()}:${selector}`);

    return {
      acir,
      partialWitness,
      callStackItem,
      returnValues,
      readRequestCommitmentIndices,
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
