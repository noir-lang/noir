import {
  CallContext,
  ContractDeploymentData,
  FunctionData,
  PrivateCallStackItem,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { FunctionSelector, decodeReturnValues } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { to2Fields } from '@aztec/foundation/serialize';
import { FunctionL2Logs, NotePreimage, NoteSpendingInfo } from '@aztec/types';

import { extractPrivateCircuitPublicInputs, frToAztecAddress } from '../acvm/deserialize.js';
import {
  ZERO_ACVM_FIELD,
  acvm,
  convertACVMFieldToBuffer,
  extractCallStack,
  fromACVMField,
  toACVMField,
  toACVMWitness,
  toAcvmCallPrivateStackItem,
  toAcvmEnqueuePublicFunctionResult,
} from '../acvm/index.js';
import { ExecutionError } from '../common/errors.js';
import {
  AcirSimulator,
  ExecutionResult,
  FunctionAbiWithDebugMetadata,
  NewNoteData,
  NewNullifierData,
} from '../index.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { acvmFieldMessageToString, oracleDebugCallToFormattedStr } from './debug.js';

/**
 * The private function execution class.
 */
export class PrivateFunctionExecution {
  constructor(
    private context: ClientTxExecutionContext,
    private abi: FunctionAbiWithDebugMetadata,
    private contractAddress: AztecAddress,
    private functionData: FunctionData,
    private argsHash: Fr,
    private callContext: CallContext,
    private curve: Grumpkin,
    private sideEffectCounter: number = 0,
    private log = createDebugLogger('aztec:simulator:secret_execution'),
  ) {}

  /**
   * Executes the function.
   * @returns The execution result.
   */
  public async run(): Promise<ExecutionResult> {
    const selector = this.functionData.selector;
    this.log(`Executing external function ${this.contractAddress}:${selector}`);

    const acir = Buffer.from(this.abi.bytecode, 'base64');
    const initialWitness = this.getInitialWitness();

    // TODO: Move to ClientTxExecutionContext.
    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];
    const enqueuedPublicFunctionCalls: PublicCallRequest[] = [];
    const encryptedLogs = new FunctionL2Logs([]);
    const unencryptedLogs = new FunctionL2Logs([]);

    const { partialWitness } = await acvm(await AcirSimulator.getSolver(), acir, initialWitness, {
      packArguments: async args => {
        return toACVMField(await this.context.packedArgsCache.pack(args.map(fromACVMField)));
      },
      getSecretKey: ([ownerX], [ownerY]) => this.context.getSecretKey(this.contractAddress, ownerX, ownerY),
      getPublicKey: async ([acvmAddress]) => {
        const address = frToAztecAddress(fromACVMField(acvmAddress));
        const { publicKey, partialAddress } = await this.context.db.getCompleteAddress(address);
        return [publicKey.x, publicKey.y, partialAddress].map(toACVMField);
      },
      getNotes: ([slot], [numSelects], selectBy, selectValues, sortBy, sortOrder, [limit], [offset], [returnSize]) =>
        this.context.getNotes(
          this.contractAddress,
          slot,
          +numSelects,
          selectBy,
          selectValues,
          sortBy,
          sortOrder,
          +limit,
          +offset,
          +returnSize,
        ),
      getRandomField: () => Promise.resolve(toACVMField(Fr.random())),
      notifyCreatedNote: ([storageSlot], preimage, [innerNoteHash]) => {
        this.context.pushNewNote(
          this.contractAddress,
          fromACVMField(storageSlot),
          preimage.map(f => fromACVMField(f)),
          fromACVMField(innerNoteHash),
        );

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1040): remove newNotePreimages
        // as it is redundant with pendingNoteData. Consider renaming pendingNoteData->pendingNotePreimages.
        newNotePreimages.push({
          storageSlot: fromACVMField(storageSlot),
          preimage: preimage.map(f => fromACVMField(f)),
        });
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      notifyNullifiedNote: async ([slot], [nullifier], acvmPreimage, [innerNoteHash]) => {
        newNullifiers.push({
          preimage: acvmPreimage.map(f => fromACVMField(f)),
          storageSlot: fromACVMField(slot),
          nullifier: fromACVMField(nullifier),
        });
        await this.context.pushNewNullifier(fromACVMField(nullifier), this.contractAddress);
        this.context.nullifyPendingNotes(fromACVMField(innerNoteHash), this.contractAddress, fromACVMField(slot));
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      callPrivateFunction: async ([acvmContractAddress], [acvmFunctionSelector], [acvmArgsHash]) => {
        const contractAddress = fromACVMField(acvmContractAddress);
        const functionSelector = fromACVMField(acvmFunctionSelector);
        this.log(
          `Calling private function ${contractAddress.toString()}:${functionSelector} from ${this.callContext.storageContractAddress.toString()}`,
        );

        const childExecutionResult = await this.callPrivateFunction(
          frToAztecAddress(contractAddress),
          FunctionSelector.fromField(functionSelector),
          fromACVMField(acvmArgsHash),
          this.callContext,
          this.curve,
        );

        nestedExecutionContexts.push(childExecutionResult);

        return toAcvmCallPrivateStackItem(childExecutionResult.callStackItem);
      },
      getL1ToL2Message: ([msgKey]) => {
        return this.context.getL1ToL2Message(fromACVMField(msgKey));
      },
      getCommitment: ([commitment]) => this.context.getCommitment(this.contractAddress, commitment),
      debugLog: (...args) => {
        this.log(oracleDebugCallToFormattedStr(args));
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      debugLogWithPrefix: (arg0, ...args) => {
        this.log(`${acvmFieldMessageToString(arg0)}: ${oracleDebugCallToFormattedStr(args)}`);
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      enqueuePublicFunctionCall: async ([acvmContractAddress], [acvmFunctionSelector], [acvmArgsHash]) => {
        const enqueuedRequest = await this.enqueuePublicFunctionCall(
          frToAztecAddress(fromACVMField(acvmContractAddress)),
          FunctionSelector.fromField(fromACVMField(acvmFunctionSelector)),
          this.context.packedArgsCache.unpack(fromACVMField(acvmArgsHash)),
          this.callContext,
        );

        this.log(
          `Enqueued call to public function (with side-effect counter #${enqueuedRequest.sideEffectCounter}) ${acvmContractAddress}:${acvmFunctionSelector}`,
        );
        enqueuedPublicFunctionCalls.push(enqueuedRequest);
        return toAcvmEnqueuePublicFunctionResult(enqueuedRequest);
      },
      emitUnencryptedLog: message => {
        // https://github.com/AztecProtocol/aztec-packages/issues/885
        const log = Buffer.concat(message.map(charBuffer => convertACVMFieldToBuffer(charBuffer).subarray(-1)));
        unencryptedLogs.logs.push(log);
        this.log(`Emitted unencrypted log: "${log.toString('ascii')}"`);
        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      emitEncryptedLog: ([acvmContractAddress], [acvmStorageSlot], [encPubKeyX], [encPubKeyY], acvmPreimage) => {
        const contractAddress = AztecAddress.fromBuffer(convertACVMFieldToBuffer(acvmContractAddress));
        const storageSlot = fromACVMField(acvmStorageSlot);
        const preimage = acvmPreimage.map(f => fromACVMField(f));

        const notePreimage = new NotePreimage(preimage);
        const noteSpendingInfo = new NoteSpendingInfo(notePreimage, contractAddress, storageSlot);
        const ownerPublicKey = new Point(fromACVMField(encPubKeyX), fromACVMField(encPubKeyY));

        const encryptedNotePreimage = noteSpendingInfo.toEncryptedBuffer(ownerPublicKey, this.curve);

        encryptedLogs.logs.push(encryptedNotePreimage);

        return Promise.resolve(ZERO_ACVM_FIELD);
      },
      getPortalContractAddress: async ([aztecAddress]) => {
        const contractAddress = AztecAddress.fromString(aztecAddress);
        const portalContactAddress = await this.context.db.getPortalContractAddress(contractAddress);
        return Promise.resolve(toACVMField(portalContactAddress));
      },
    }).catch((err: Error) => {
      throw new ExecutionError(
        err.message,
        {
          contractAddress: this.contractAddress,
          functionSelector: selector,
        },
        extractCallStack(err, this.abi.debug),
        { cause: err },
      );
    });

    const publicInputs = extractPrivateCircuitPublicInputs(partialWitness, acir);

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165) --> set this in Noir
    publicInputs.encryptedLogsHash = to2Fields(encryptedLogs.hash());
    publicInputs.encryptedLogPreimagesLength = new Fr(encryptedLogs.getSerializedLength());
    publicInputs.unencryptedLogsHash = to2Fields(unencryptedLogs.hash());
    publicInputs.unencryptedLogPreimagesLength = new Fr(unencryptedLogs.getSerializedLength());

    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);
    const returnValues = decodeReturnValues(this.abi, publicInputs.returnValues);

    this.log(`Returning from call to ${this.contractAddress.toString()}:${selector}`);

    const readRequestPartialWitnesses = this.context.getReadRequestPartialWitnesses();

    return {
      acir,
      partialWitness,
      callStackItem,
      returnValues,
      readRequestPartialWitnesses,
      preimages: {
        newNotes: newNotePreimages,
        nullifiedNotes: newNullifiers,
      },
      vk: Buffer.from(this.abi.verificationKey!, 'hex'),
      nestedExecutions: nestedExecutionContexts,
      enqueuedPublicFunctionCalls,
      encryptedLogs,
      unencryptedLogs,
    };
  }

  // We still need this function until we can get user-defined ordering of structs for fn arguments
  // TODO When that is sorted out on noir side, we can use instead the utilities in serialize.ts
  /**
   * Writes the function inputs to the initial witness.
   * @returns The initial witness.
   */
  private getInitialWitness() {
    const contractDeploymentData = this.context.txContext.contractDeploymentData ?? ContractDeploymentData.empty();

    const blockData = this.context.historicBlockData;

    const fields = [
      this.callContext.msgSender,
      this.callContext.storageContractAddress,
      this.callContext.portalContractAddress,
      this.callContext.isDelegateCall,
      this.callContext.isStaticCall,
      this.callContext.isContractDeployment,

      ...blockData.toArray(),

      contractDeploymentData.deployerPublicKey.x,
      contractDeploymentData.deployerPublicKey.y,
      contractDeploymentData.constructorVkHash,
      contractDeploymentData.functionTreeRoot,
      contractDeploymentData.contractAddressSalt,
      contractDeploymentData.portalContractAddress,

      this.context.txContext.chainId,
      this.context.txContext.version,

      ...this.context.packedArgsCache.unpack(this.argsHash),
    ];

    return toACVMWitness(1, fields);
  }

  /**
   * Calls a private function as a nested execution.
   * @param targetContractAddress - The address of the contract to call.
   * @param targetFunctionSelector - The function selector of the function to call.
   * @param targetArgsHash - The packed arguments to pass to the function.
   * @param callerContext - The call context of the caller.
   * @param curve - The curve instance to use for elliptic curve operations.
   * @returns The execution result.
   */
  private async callPrivateFunction(
    targetContractAddress: AztecAddress,
    targetFunctionSelector: FunctionSelector,
    targetArgsHash: Fr,
    callerContext: CallContext,
    curve: Grumpkin,
  ) {
    const targetAbi = await this.context.db.getFunctionABI(targetContractAddress, targetFunctionSelector);
    const targetFunctionData = FunctionData.fromAbi(targetAbi);
    const derivedCallContext = await this.deriveCallContext(callerContext, targetContractAddress, false, false);
    const context = this.context.extend();

    const nestedExecution = new PrivateFunctionExecution(
      context,
      targetAbi,
      targetContractAddress,
      targetFunctionData,
      targetArgsHash,
      derivedCallContext,
      curve,
      this.sideEffectCounter,
      this.log,
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
    targetFunctionSelector: FunctionSelector,
    targetArgs: Fr[],
    callerContext: CallContext,
  ): Promise<PublicCallRequest> {
    const targetAbi = await this.context.db.getFunctionABI(targetContractAddress, targetFunctionSelector);
    const derivedCallContext = await this.deriveCallContext(callerContext, targetContractAddress, false, false);

    return PublicCallRequest.from({
      args: targetArgs,
      callContext: derivedCallContext,
      functionData: FunctionData.fromAbi(targetAbi),
      contractAddress: targetContractAddress,
      sideEffectCounter: this.sideEffectCounter++, // update after assigning current value to call
    });

    // TODO($846): if enqueued public calls are associated with global
    // side-effect counter, that will leak info about how many other private
    // side-effects occurred in the TX. Ultimately the private kernel should
    // just output everything in the proper order without any counters.
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
