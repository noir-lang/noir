import { ACVMField, ACVMWitness, fromACVMField } from './acvm.js';
import { AztecAddress, Fr, EthAddress } from '@aztec/foundation';
import {
  ARGS_LENGTH,
  CallContext,
  ContractDeploymentData,
  EMITTED_EVENTS_LENGTH,
  L1_MSG_STACK_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  PrivateCircuitPublicInputs,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  RETURN_VALUES_LENGTH,
} from '@aztec/circuits.js';
import { select_return_flattened as selectPublicWitnessFlattened } from '@noir-lang/noir_util_wasm';

// Utilities to read TS classes from ACVM Field arrays
// In the order that the ACVM provides them

export function frToAztecAddress(fr: Fr): AztecAddress {
  return new AztecAddress(fr.toBuffer());
}

export function frToNumber(fr: Fr): number {
  return Number(fr.value);
}

export function frToEthAddress(fr: Fr): EthAddress {
  return new EthAddress(fr.toBuffer().slice(-EthAddress.SIZE_IN_BYTES));
}

export function frToBoolean(fr: Fr): boolean {
  const buf = fr.toBuffer();
  return buf[buf.length - 1] !== 0;
}

export function frToSelector(fr: Fr): Buffer {
  return fr.toBuffer().slice(-4);
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
