import {
  ACVMField,
  acvm,
  toACVMField,
  fromACVMField,
  ZERO_ACVM_FIELD,
  toAcvmCallPrivateStackItem,
  toACVMWitness,
} from '../acvm/index.js';
import { CallContext, PrivateCallStackItem, FunctionData } from '@aztec/circuits.js';
import { extractPublicInputs, frToAztecAddress, frToSelector } from '../acvm/deserialize.js';
import { FunctionAbi } from '@aztec/foundation/abi';
import { createDebugLogger } from '@aztec/foundation/log';
import { decodeReturnValues } from '../abi_coder/decoder.js';
import { ClientTxExecutionContext } from './client_execution_context.js';
import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { sizeOfType } from '../index.js';

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
    this.log(
      `Executing external function ${this.contractAddress.toShortString()}:${this.functionData.functionSelector.toString(
        'hex',
      )}`,
    );

    const acir = Buffer.from(this.abi.bytecode, 'hex');
    const initialWitness = this.writeInputs();

    const newNotePreimages: NewNoteData[] = [];
    const newNullifiers: NewNullifierData[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];

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
      viewNotesPage: notAvailable,
      storageRead: notAvailable,
      storageWrite: notAvailable,
      callPublicFunction: notAvailable,
    });

    const publicInputs = extractPublicInputs(partialWitness, acir);

    const callStackItem = new PrivateCallStackItem(this.contractAddress, this.functionData, publicInputs);

    const returnValues = decodeReturnValues(this.abi, publicInputs.returnValues);

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
    const targetPortalContractAddress = await this.context.db.getPortalContractAddress(targetContractAddress);
    const targetFunctionData = new FunctionData(targetFunctionSelector, true, false);
    const derivedCallContext = this.deriveCallContext(
      callerContext,
      targetContractAddress,
      targetPortalContractAddress,
      false,
      false,
    );

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
   * Derives the call context for a nested execution.
   * @param parentContext - The parent call context.
   * @param targetContractAddress - The address of the contract being called.
   * @param portalContractAddress - The address of the portal contract.
   * @param isDelegateCall - Whether the call is a delegate call.
   * @param isStaticCall - Whether the call is a static call.
   * @returns The derived call context.
   */
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
