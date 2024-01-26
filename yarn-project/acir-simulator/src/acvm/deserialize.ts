import {
  BlockHeader,
  CallContext,
  ContractDeploymentData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MAX_READ_REQUESTS_PER_CALL,
  NUM_FIELDS_PER_SHA256,
  NullifierKeyValidationRequest,
  PrivateCircuitPublicInputs,
  PublicCircuitPublicInputs,
  RETURN_VALUES_LENGTH,
  SideEffect,
  SideEffectLinkedToNoteHash,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { FieldReader, Tuple } from '@aztec/foundation/serialize';

import { getReturnWitness } from '@noir-lang/acvm_js';

import { ACVMField, ACVMWitness } from './acvm_types.js';

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: ACVMField): Fr {
  return Fr.fromBuffer(Buffer.from(field.slice(2), 'hex'));
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
 * Extracts the return fields of a given partial witness.
 * @param acir - The bytecode of the function.
 * @param partialWitness - The witness to extract from.
 * @returns The return values.
 */
export function extractReturnWitness(acir: Buffer, partialWitness: ACVMWitness): Fr[] {
  const returnWitness = getReturnWitness(acir, partialWitness);
  const sortedKeys = [...returnWitness.keys()].sort((a, b) => a - b);
  return sortedKeys.map(key => returnWitness.get(key)!).map(fromACVMField);
}

/**
 * Create a reader for the public inputs of the ACVM generated partial witness.
 */
function createPublicInputsReader(witness: ACVMWitness, acir: Buffer) {
  const fields = extractReturnWitness(acir, witness);
  return new FieldReader(fields);
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
  const witnessReader = createPublicInputsReader(partialWitness, acir);

  const callContext = witnessReader.readObject(CallContext);
  const argsHash = witnessReader.readField();
  const returnValues = witnessReader.readFieldArray(RETURN_VALUES_LENGTH);
  const readRequests = witnessReader.readArray(MAX_READ_REQUESTS_PER_CALL, SideEffect);
  const nullifierKeyValidationRequests = witnessReader.readArray(
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL,
    NullifierKeyValidationRequest,
  );
  const newCommitments = witnessReader.readArray(MAX_NEW_COMMITMENTS_PER_CALL, SideEffect);
  const newNullifiers = witnessReader.readArray(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash);
  const privateCallStack = witnessReader.readFieldArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL);
  const publicCallStack = witnessReader.readFieldArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
  const newL2ToL1Msgs = witnessReader.readFieldArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL);
  const endSideEffectCounter = witnessReader.readField();

  const encryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const unencryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const encryptedLogPreimagesLength = witnessReader.readField();
  const unencryptedLogPreimagesLength = witnessReader.readField();

  const blockHeader = new BlockHeader(
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    Fr.ZERO, // TODO(#3441)
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
    nullifierKeyValidationRequests,
    newCommitments,
    newNullifiers,
    privateCallStack,
    publicCallStack,
    newL2ToL1Msgs,
    endSideEffectCounter,
    encryptedLogsHash,
    unencryptedLogsHash,
    encryptedLogPreimagesLength,
    unencryptedLogPreimagesLength,
    blockHeader,
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
  const witnessReader = createPublicInputsReader(partialWitness, acir);

  const callContext = witnessReader.readObject(CallContext);

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
  const newCommitments = witnessReader.readArray(MAX_NEW_COMMITMENTS_PER_CALL, SideEffect);
  const newNullifiers = witnessReader.readArray(MAX_NEW_NULLIFIERS_PER_CALL, SideEffectLinkedToNoteHash);
  const newL2ToL1Msgs = witnessReader.readFieldArray(MAX_NEW_L2_TO_L1_MSGS_PER_CALL);

  const unencryptedLogsHash = witnessReader.readFieldArray(NUM_FIELDS_PER_SHA256);
  const unencryptedLogPreimagesLength = witnessReader.readField();

  const blockHeader = new BlockHeader(
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    witnessReader.readField(),
    Fr.ZERO, // TODO(#3441)
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
    blockHeader,
    proverAddress,
  );
}
