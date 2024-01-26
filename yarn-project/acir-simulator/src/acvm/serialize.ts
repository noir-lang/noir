import {
  BlockHeader,
  CallContext,
  ContractDeploymentData,
  FunctionData,
  GlobalVariables,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { ACVMField } from './acvm_types.js';
import { MessageLoadOracleInputs } from './oracle/typed_oracle.js';

/**
 * Adapts the buffer to the field size.
 * @param originalBuf - The buffer to adapt.
 * @returns The adapted buffer.
 */
function adaptBufferSize(originalBuf: Buffer) {
  const buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
  if (originalBuf.length > buffer.length) {
    throw new Error('Buffer does not fit in field');
  }
  originalBuf.copy(buffer, buffer.length - originalBuf.length);
  return buffer;
}

/**
 * Converts a value to an ACVM field.
 * @param value - The value to convert.
 * @returns The ACVM field.
 */
export function toACVMField(
  value: AztecAddress | EthAddress | Fr | Buffer | boolean | number | bigint | ACVMField,
): ACVMField {
  let buffer;
  if (Buffer.isBuffer(value)) {
    buffer = value;
  } else if (typeof value === 'boolean' || typeof value === 'number' || typeof value === 'bigint') {
    buffer = new Fr(value).toBuffer();
  } else if (typeof value === 'string') {
    buffer = Fr.fromString(value).toBuffer();
  } else {
    buffer = value.toBuffer();
  }
  return `0x${adaptBufferSize(buffer).toString('hex')}`;
}

// Utilities to write TS classes to ACVM Field arrays
// In the order that the ACVM expects them

/**
 * Converts a function data to ACVM fields.
 * @param functionData - The function data to convert.
 * @returns The ACVM fields.
 */
export function toACVMFunctionData(functionData: FunctionData): ACVMField[] {
  return [
    toACVMField(functionData.selector.toBuffer()),
    toACVMField(functionData.isInternal),
    toACVMField(functionData.isPrivate),
    toACVMField(functionData.isConstructor),
  ];
}

/**
 * Converts a call context to ACVM fields.
 * @param callContext - The call context to convert.
 * @returns The ACVM fields.
 */
export function toACVMCallContext(callContext: CallContext): ACVMField[] {
  return [
    toACVMField(callContext.msgSender),
    toACVMField(callContext.storageContractAddress),
    toACVMField(callContext.portalContractAddress),
    toACVMField(callContext.functionSelector.toField()),
    toACVMField(callContext.isDelegateCall),
    toACVMField(callContext.isStaticCall),
    toACVMField(callContext.isContractDeployment),
    toACVMField(callContext.startSideEffectCounter),
  ];
}

/**
 * Converts a contract deployment data to ACVM fields.
 * @param contractDeploymentData - The contract deployment data to convert.
 * @returns The ACVM fields.
 */
export function toACVMContractDeploymentData(contractDeploymentData: ContractDeploymentData): ACVMField[] {
  return [
    toACVMField(contractDeploymentData.deployerPublicKey.x),
    toACVMField(contractDeploymentData.deployerPublicKey.y),
    toACVMField(contractDeploymentData.constructorVkHash),
    toACVMField(contractDeploymentData.functionTreeRoot),
    toACVMField(contractDeploymentData.contractAddressSalt),
    toACVMField(contractDeploymentData.portalContractAddress),
  ];
}

/**
 * Converts a block header into ACVM fields.
 * @param blockHeader - The block header object to convert.
 * @returns The ACVM fields.
 */
export function toACVMBlockHeader(blockHeader: BlockHeader): ACVMField[] {
  return [
    toACVMField(blockHeader.noteHashTreeRoot),
    toACVMField(blockHeader.nullifierTreeRoot),
    toACVMField(blockHeader.contractTreeRoot),
    toACVMField(blockHeader.l1ToL2MessageTreeRoot),
    toACVMField(blockHeader.archiveRoot),
    toACVMField(blockHeader.publicDataTreeRoot),
    toACVMField(blockHeader.globalVariablesHash),
  ];
}

/**
 * Converts global variables into ACVM fields
 * @param globalVariables - The global variables object to convert.
 * @returns The ACVM fields
 */
export function toACVMGlobalVariables(globalVariables: GlobalVariables): ACVMField[] {
  return [
    toACVMField(globalVariables.chainId),
    toACVMField(globalVariables.version),
    toACVMField(globalVariables.blockNumber),
    toACVMField(globalVariables.timestamp),
  ];
}

/**
 * Converts the public inputs structure to ACVM fields.
 * @param publicInputs - The public inputs to convert.
 * @returns The ACVM fields.
 */
export function toACVMPublicInputs(publicInputs: PrivateCircuitPublicInputs): ACVMField[] {
  return [
    ...toACVMCallContext(publicInputs.callContext),
    toACVMField(publicInputs.argsHash),

    ...publicInputs.returnValues.map(toACVMField),
    ...publicInputs.readRequests.flatMap(x => x.toFields()).map(toACVMField),
    ...publicInputs.nullifierKeyValidationRequests.flatMap(x => x.toFields()).map(toACVMField),
    ...publicInputs.newCommitments.flatMap(x => x.toFields()).map(toACVMField),
    ...publicInputs.newNullifiers.flatMap(x => x.toFields()).map(toACVMField),
    ...publicInputs.privateCallStackHashes.map(toACVMField),
    ...publicInputs.publicCallStackHashes.map(toACVMField),
    ...publicInputs.newL2ToL1Msgs.map(toACVMField),
    toACVMField(publicInputs.endSideEffectCounter),
    ...publicInputs.encryptedLogsHash.map(toACVMField),
    ...publicInputs.unencryptedLogsHash.map(toACVMField),

    toACVMField(publicInputs.encryptedLogPreimagesLength),
    toACVMField(publicInputs.unencryptedLogPreimagesLength),

    ...toACVMBlockHeader(publicInputs.blockHeader),

    ...toACVMContractDeploymentData(publicInputs.contractDeploymentData),

    toACVMField(publicInputs.chainId),
    toACVMField(publicInputs.version),
  ];
}

/**
 * Converts a private call stack item to ACVM fields.
 * @param item - The private call stack item to convert.
 * @returns The ACVM fields.
 */
export function toAcvmCallPrivateStackItem(item: PrivateCallStackItem): ACVMField[] {
  return [
    toACVMField(item.contractAddress),
    ...toACVMFunctionData(item.functionData),
    ...toACVMPublicInputs(item.publicInputs),
    toACVMField(item.isExecutionRequest),
  ];
}

/**
 * Converts a public call stack item with the request for executing a public function to
 * a set of ACVM fields accepted by the enqueue_public_function_call_oracle Aztec.nr function.
 * Note that only the fields related to the request are serialized: those related to the result
 * are empty since this is just an execution request, so we don't send them to the circuit.
 * @param item - The public call stack item to serialize to be passed onto Noir.
 * @returns The fields expected by the enqueue_public_function_call_oracle Aztec.nr function.
 */
export function toAcvmEnqueuePublicFunctionResult(item: PublicCallRequest): ACVMField[] {
  return [
    toACVMField(item.contractAddress),
    ...toACVMFunctionData(item.functionData),
    ...toACVMCallContext(item.callContext),
    toACVMField(item.getArgsHash()),
  ];
}

/**
 * Converts the result of loading messages to ACVM fields.
 * @param messageLoadOracleInputs - The result of loading messages to convert.
 * @returns The Message Oracle Fields.
 */
export function toAcvmL1ToL2MessageLoadOracleInputs(messageLoadOracleInputs: MessageLoadOracleInputs): ACVMField[] {
  return [
    ...messageLoadOracleInputs.message.map(f => toACVMField(f)),
    toACVMField(messageLoadOracleInputs.index),
    ...messageLoadOracleInputs.siblingPath.map(f => toACVMField(f)),
  ];
}

/**
 * Inserts a list of ACVM fields to a witness.
 * @param witnessStartIndex - The index where to start inserting the fields.
 * @param fields - The fields to insert.
 * @returns The witness.
 */
export function toACVMWitness(witnessStartIndex: number, fields: Parameters<typeof toACVMField>[0][]) {
  return fields.reduce((witness, field, index) => {
    witness.set(index + witnessStartIndex, toACVMField(field));
    return witness;
  }, new Map<number, ACVMField>());
}
