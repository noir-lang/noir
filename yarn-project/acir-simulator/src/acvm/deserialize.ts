import {
  CallContext,
  ContractDeploymentData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FunctionSelector,
  HistoricBlockData,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  NUM_FIELDS_PER_SHA256,
  PrivateCircuitPublicInputs,
  PublicCircuitPublicInputs,
  RETURN_VALUES_LENGTH,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { Tuple } from '@aztec/foundation/serialize';

import { getReturnWitness } from '@noir-lang/acvm_js';

import { ACVMField, ACVMWitness } from './acvm.js';

/**
 * Converts an ACVM field to a Buffer.
 * @param field - The ACVM field to convert.
 * @returns The Buffer.
 */
export function convertACVMFieldToBuffer(field: ACVMField): Buffer {
  return Buffer.from(field.slice(2), 'hex');
}

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: ACVMField): Fr {
  return Fr.fromBuffer(convertACVMFieldToBuffer(field));
}

// Utilities to read TS classes from ACVM Field arrays
// In the order that the ACVM provides them

/**
 * Converts a field to an Aztec address.
 * @param fr - The field to convert.
 * @returns The Aztec address.
 */
export function frToAztecAddress(fr: Fr): AztecAddress {
  return new AztecAddress(fr.toBuffer());
}

/**
 * Converts a field to a number.
 * @param fr - The field to convert.
 * @returns The number.
 */
export function frToNumber(fr: Fr): number {
  return Number(fr.value);
}

/**
 * Converts a field to a boolean.
 * @param fr - The field to convert.
 * @returns The boolean.
 */
export function frToBoolean(fr: Fr): boolean {
  const buf = fr.toBuffer();
  return buf[buf.length - 1] !== 0;
}

/**
 * Extracts the return fields of a given partial witness.
 * @param acir - The bytecode of the function.
 * @param partialWitness - The witness to extract from.
 * @returns The return values.
 */
export function extractReturnWitness(acir: Buffer, partialWitness: ACVMWitness): ACVMField[] {
  const returnWitness = getReturnWitness(acir, partialWitness);
  const sortedKeys = [...returnWitness.keys()].sort((a, b) => a - b);
  return sortedKeys.map(key => returnWitness.get(key)!);
}

/**
 * A utility reader for the public inputs of the ACVM generated partial witness.
 */
export class PublicInputsReader {
  private publicInputs: ACVMField[];

  constructor(witness: ACVMWitness, acir: Buffer) {
    this.publicInputs = extractReturnWitness(acir, witness);
  }

  /**
   * Reads a field from the public inputs.
   * @returns The field.
   */
  public readField(): Fr {
    const acvmField = this.publicInputs.shift();
    if (!acvmField) throw new Error('Not enough public inputs');
    return fromACVMField(acvmField);
  }

  /**
   * Reads an array of fields from the public inputs.
   * @param length - The length of the array.
   * @returns The array of fields.
   */
  public readFieldArray<N extends number>(length: N): Tuple<Fr, N> {
    const array: Fr[] = [];
    for (let i = 0; i < length; i++) {
      array.push(this.readField());
    }
    return array as Tuple<Fr, N>;
  }
}

/**
 * Extracts the public inputs from the ACVM generated partial witness.
 * @param partialWitness - The partial witness.
 * @param acir - The ACIR bytecode.
 * @returns The public inputs.
 */
export function extractPrivateCircuitPublicInputs(
  partialWitness: ACVMWitness,
  acir: Buffer,
): PrivateCircuitPublicInputs {
  const witnessReader = new PublicInputsReader(partialWitness, acir);

  const callContext = new CallContext(
    frToAztecAddress(witnessReader.readField()),
    frToAztecAddress(witnessReader.readField()),
    witnessReader.readField(),
    FunctionSelector.fromField(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
  );

  const argsHash = witnessReader.readField();
  const returnValues = witnessReader.readFieldArray(RETURN_VALUES_LENGTH);
  const readRequests = witnessReader.readFieldArray(MAX_READ_REQUESTS_PER_CALL);
  const newCommitments = witnessReader.readFieldArray(MAX_NEW_COMMITMENTS_PER_CALL);
  const newNullifiers = witnessReader.readFieldArray(MAX_NEW_NULLIFIERS_PER_CALL);
  const nullifiedCommitments = witnessReader.readFieldArray(MAX_NEW_NULLIFIERS_PER_CALL);
  const privateCallStack = witnessReader.readFieldArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
  const publicCallStack = witnessReader.readFieldArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
  const newL2ToL1Msgs = witnessReader.readFieldArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL);

  const encryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const unencryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const encryptedLogPreimagesLength = witnessReader.readField();
  const unencryptedLogPreimagesLength = witnessReader.readField();

  const historicBlockData = new HistoricBlockData(
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    Fr.ZERO,
    witnessReader.readField(),
    witnessReader.readField(),
  );

  const contractDeploymentData = new ContractDeploymentData(
    new Point(witnessReader.readField(), witnessReader.readField()),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    EthAddress.fromField(witnessReader.readField()),
  );

  const chainId = witnessReader.readField();
  const version = witnessReader.readField();

  return new PrivateCircuitPublicInputs(
    callContext,
    argsHash,
    returnValues,
    readRequests,
    newCommitments,
    newNullifiers,
    nullifiedCommitments,
    privateCallStack,
    publicCallStack,
    newL2ToL1Msgs,
    encryptedLogsHash,
    unencryptedLogsHash,
    encryptedLogPreimagesLength,
    unencryptedLogPreimagesLength,
    historicBlockData,
    contractDeploymentData,
    chainId,
    version,
  );
}

/**
 * Extracts the public circuit public inputs from the ACVM generated partial witness.
 * @param partialWitness - The partial witness.
 * @param acir - The ACIR bytecode.
 * @returns The public inputs.
 */
export function extractPublicCircuitPublicInputs(partialWitness: ACVMWitness, acir: Buffer): PublicCircuitPublicInputs {
  const witnessReader = new PublicInputsReader(partialWitness, acir);

  const callContext = new CallContext(
    frToAztecAddress(witnessReader.readField()),
    frToAztecAddress(witnessReader.readField()),
    witnessReader.readField(),
    FunctionSelector.fromField(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
    frToBoolean(witnessReader.readField()),
  );

  const argsHash = witnessReader.readField();
  const returnValues = witnessReader.readFieldArray(RETURN_VALUES_LENGTH);

  const contractStorageUpdateRequests = new Array(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL).fill(
    ContractStorageUpdateRequest.empty(),
  );
  for (let i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL; i++) {
    const request = new ContractStorageUpdateRequest(
      witnessReader.readField(),
      witnessReader.readField(),
      witnessReader.readField(),
    );
    contractStorageUpdateRequests[i] = request;
  }
  const contractStorageReads = new Array(MAX_PUBLIC_DATA_READS_PER_CALL).fill(ContractStorageRead.empty());
  for (let i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL; i++) {
    const request = new ContractStorageRead(witnessReader.readField(), witnessReader.readField());
    contractStorageReads[i] = request;
  }

  const publicCallStack = witnessReader.readFieldArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
  const newCommitments = witnessReader.readFieldArray(MAX_NEW_COMMITMENTS_PER_CALL);
  const newNullifiers = witnessReader.readFieldArray(MAX_NEW_NULLIFIERS_PER_CALL);
  const newL2ToL1Msgs = witnessReader.readFieldArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL);

  const unencryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const unencryptedLogPreimagesLength = witnessReader.readField();

  const historicBlockData = new HistoricBlockData(
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    Fr.ZERO,
    witnessReader.readField(),
    witnessReader.readField(),
  );
  const proverAddress = AztecAddress.fromField(witnessReader.readField());

  return new PublicCircuitPublicInputs(
    callContext,
    argsHash,
    returnValues,
    contractStorageUpdateRequests as Tuple<
      ContractStorageUpdateRequest,
      typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL
    >,
    contractStorageReads as Tuple<ContractStorageRead, typeof MAX_PUBLIC_DATA_READS_PER_CALL>,
    publicCallStack,
    newCommitments,
    newNullifiers,
    newL2ToL1Msgs,
    unencryptedLogsHash,
    unencryptedLogPreimagesLength,
    historicBlockData,
    proverAddress,
  );
}
