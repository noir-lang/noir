import { ACVMField, acvmMock, ACVMWitness, toACVMField, fromACVMField } from './acvm.js';
import {
  ARGS_LENGTH,
  AztecAddress,
  CallContext,
  ContractDeploymentData,
  EMITTED_EVENTS_LENGTH,
  EthAddress,
  Fr,
  L1_MSG_STACK_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  OldTreeRoots,
  PrivateCircuitPublicInputs,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
  TxRequest,
} from '@aztec/circuits.js';
import { DBOracle, PrivateCallStackItem } from './db_oracle.js';
import { frToAztecAddress, frToBoolean, frToEthAddress, WitnessReader, WitnessWriter } from './witness_io.js';
import { randomBytes } from '@aztec/foundation';
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
      1, // TODO this comes from contract ABI
      this.request.functionData.functionSelector,
      this.request.args,
      callContext,
    );
  }

  // Separate function so we can recurse in the future
  private async runExternalFunction(
    abi: FunctionAbi,
    contractAddress: AztecAddress,
    witnessStartIndex: number,
    functionSelector: number,
    args: Fr[],
    callContext: CallContext,
  ): Promise<ExecutionResult> {
    const acir = Buffer.from(abi.bytecode, 'hex');
    const initialWitness = this.arrangeInitialWitness(args, callContext, witnessStartIndex);
    const newNotePreimages: Array<{ preimage: Fr[]; storageSlot: Fr }> = [];
    const newNullifiers: Fr[] = [];
    const nestedExecutionContexts: ExecutionResult[] = [];

    const { partialWitness } = await acvmMock(acir, initialWitness, {
      getSecretKey: ([address]: ACVMField[]) => {
        return this.getSecretKey(contractAddress, address);
      },
      getNotes2: async ([storageSlot]: ACVMField[]) => {
        return await this.getNotes(contractAddress, storageSlot, 2);
      },
      getRandomField: () => Promise.resolve([toACVMField(Fr.fromBuffer(randomBytes(Fr.SIZE_IN_BYTES)))]),
      notifyCreatedNote: (params: ACVMField[]) => {
        const [storageSlot, ...acvmPreimage] = params;
        const preimage = acvmPreimage.map(f => fromACVMField(f));
        newNotePreimages.push({
          preimage,
          storageSlot: fromACVMField(storageSlot),
        });
        return Promise.resolve([]);
      },
      notifyNullifiedNote: ([nullifier]: ACVMField[]) => {
        newNullifiers.push(fromACVMField(nullifier));
        return Promise.resolve([]);
      },
    });

    const publicInputs = this.extractPublicInputs(partialWitness, witnessStartIndex);

    const callStackItem = new PrivateCallStackItem(contractAddress, functionSelector, publicInputs);

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

  private arrangeInitialWitness(args: Fr[], callContext: CallContext, witnessStartIndex: number) {
    const witness: ACVMWitness = new Map();

    const writer = new WitnessWriter(witnessStartIndex, witness);

    writer.writeField(callContext.msgSender);
    writer.writeField(callContext.storageContractAddress);
    writer.writeField(callContext.portalContractAddress);
    writer.writeField(callContext.isDelegateCall);
    writer.writeField(callContext.isStaticCall);
    writer.writeField(callContext.isContractDeployment);

    writer.writeFieldArray(
      new Array(ARGS_LENGTH).fill(Fr.fromBuffer(Buffer.alloc(Fr.SIZE_IN_BYTES))).map((value, i) => args[i] || value),
    );

    writer.writeFieldArray(new Array(RETURN_VALUES_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(EMITTED_EVENTS_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(NEW_COMMITMENTS_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(NEW_NULLIFIERS_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(PRIVATE_CALL_STACK_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(PUBLIC_CALL_STACK_LENGTH).fill(new Fr(0n)));
    writer.writeFieldArray(new Array(L1_MSG_STACK_LENGTH).fill(new Fr(0n)));

    writer.writeField(this.oldRoots.privateDataTreeRoot);
    writer.writeField(this.oldRoots.nullifierTreeRoot);
    writer.writeField(this.oldRoots.contractTreeRoot);

    writer.writeField(this.request.txContext.contractDeploymentData.constructorVkHash);
    writer.writeField(this.request.txContext.contractDeploymentData.functionTreeRoot);
    writer.writeField(this.request.txContext.contractDeploymentData.contractAddressSalt);
    writer.writeField(this.request.txContext.contractDeploymentData.portalContractAddress);

    return witness;
  }

  private extractPublicInputs(partialWitness: ACVMWitness, witnessStartIndex: number): PrivateCircuitPublicInputs {
    const witnessReader = new WitnessReader(witnessStartIndex, partialWitness);

    const callContext = new CallContext(
      frToAztecAddress(witnessReader.readField()),
      frToAztecAddress(witnessReader.readField()),
      frToEthAddress(witnessReader.readField()),
      frToBoolean(witnessReader.readField()),
      frToBoolean(witnessReader.readField()),
      frToBoolean(witnessReader.readField()),
    );

    const args = witnessReader.readFieldArray(ARGS_LENGTH);
    const returnValues = witnessReader.readFieldArray(RETURN_VALUES_LENGTH);
    const emittedEvents = witnessReader.readFieldArray(EMITTED_EVENTS_LENGTH);
    const newCommitments = witnessReader.readFieldArray(NEW_COMMITMENTS_LENGTH);
    const newNullifiers = witnessReader.readFieldArray(NEW_NULLIFIERS_LENGTH);
    const privateCallStack = witnessReader.readFieldArray(PRIVATE_CALL_STACK_LENGTH);
    const publicCallStack = witnessReader.readFieldArray(PUBLIC_CALL_STACK_LENGTH);
    const l1MsgStack = witnessReader.readFieldArray(L1_MSG_STACK_LENGTH);

    const privateDataTreeRoot = witnessReader.readField();
    const nullifierTreeRoot = witnessReader.readField();
    const contractTreeRoot = witnessReader.readField();

    const contractDeploymentData = new ContractDeploymentData(
      witnessReader.readField(),
      witnessReader.readField(),
      witnessReader.readField(),
      frToEthAddress(witnessReader.readField()),
    );

    return new PrivateCircuitPublicInputs(
      callContext,
      args,
      returnValues,
      emittedEvents,
      newCommitments,
      newNullifiers,
      privateCallStack,
      publicCallStack,
      l1MsgStack,
      privateDataTreeRoot,
      nullifierTreeRoot,
      contractTreeRoot,
      contractDeploymentData,
    );
  }
}
