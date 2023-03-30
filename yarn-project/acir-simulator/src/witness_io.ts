import { ACVMField, ACVMWitness, fromACVMField, toACVMField } from './acvm.js';
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
  TxContext,
} from '@aztec/circuits.js';
import { select_return_flattened as selectPublicWitnessFlattened } from '@noir-lang/noir_util_wasm';

export class WitnessWriter {
  constructor(private currentIndex: number, private witness: ACVMWitness) {}

  public writeField(field: Parameters<typeof toACVMField>[0]) {
    this.witness.set(this.currentIndex, toACVMField(field));
    this.currentIndex += 1;
  }

  public writeFieldArray(array: Fr[]) {
    for (const field of array) {
      this.writeField(field);
    }
  }

  public jump(amount: number) {
    this.currentIndex += amount;
  }
}

export function frToAztecAddress(fr: Fr): AztecAddress {
  return new AztecAddress(fr.toBuffer());
}

export function frToEthAddress(fr: Fr): EthAddress {
  return new EthAddress(fr.toBuffer().slice(-EthAddress.SIZE_IN_BYTES));
}

export function frToBoolean(fr: Fr): boolean {
  const buf = fr.toBuffer();
  return buf[buf.length - 1] !== 0;
}

export function writeInputs(
  args: Fr[],
  callContext: CallContext,
  txContext: TxContext,
  oldRoots: OldTreeRoots,
  witnessStartIndex = 1,
) {
  const witness: ACVMWitness = new Map();

  const writer = new WitnessWriter(witnessStartIndex, witness);

  writer.writeFieldArray(
    new Array(ARGS_LENGTH).fill(Fr.fromBuffer(Buffer.alloc(Fr.SIZE_IN_BYTES))).map((value, i) => args[i] || value),
  );

  writer.writeField(callContext.isContractDeployment);
  writer.writeField(callContext.isDelegateCall);
  writer.writeField(callContext.isStaticCall);
  writer.writeField(callContext.msgSender);
  writer.writeField(callContext.portalContractAddress);
  writer.writeField(callContext.storageContractAddress);

  writer.writeField(txContext.contractDeploymentData.constructorVkHash);
  writer.writeField(txContext.contractDeploymentData.contractAddressSalt);
  writer.writeField(txContext.contractDeploymentData.functionTreeRoot);
  writer.writeField(false);
  writer.writeField(txContext.contractDeploymentData.portalContractAddress);

  writer.writeField(oldRoots.contractTreeRoot);
  writer.writeField(oldRoots.nullifierTreeRoot);

  return witness;
}

export class WitnessReader {
  private publicInputs: ACVMField[];

  constructor(witness: ACVMWitness, acir: Buffer) {
    this.publicInputs = selectPublicWitnessFlattened(acir, witness);
  }

  public readField(): Fr {
    const acvmField = this.publicInputs.shift();
    if (!acvmField) throw new Error('Not enough public inputs');
    return fromACVMField(acvmField);
  }

  public readFieldArray(length: number): Fr[] {
    const array: Fr[] = [];
    for (let i = 0; i < length; i++) {
      array.push(this.readField());
    }
    return array;
  }
}

export function extractPublicInputs(partialWitness: ACVMWitness, acir: Buffer): PrivateCircuitPublicInputs {
  const witnessReader = new WitnessReader(partialWitness, acir);

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
