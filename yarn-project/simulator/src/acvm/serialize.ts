import { ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH, type PublicCallRequest } from '@aztec/circuits.js';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { type ACVMField } from './acvm_types.js';

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
 * Converts a public call stack item with the request for executing a public function to
 * a set of ACVM fields accepted by the enqueue_public_function_call_oracle Aztec.nr function.
 * Note that only the fields related to the request are serialized: those related to the result
 * are empty since this is just an execution request, so we don't send them to the circuit.
 * @param item - The public call stack item to serialize to be passed onto Noir.
 * @returns The fields expected by the enqueue_public_function_call_oracle Aztec.nr function.
 * TODO(#4380): Nuke this and replace it with PublicCallRequest.toFields()
 */
export function toAcvmEnqueuePublicFunctionResult(item: PublicCallRequest): ACVMField[] {
  const fields = [
    item.contractAddress.toField(),
    item.functionSelector.toField(),
    ...item.callContext.toFields(),
    item.sideEffectCounter,
    item.getArgsHash(),
  ];
  if (fields.length !== ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH) {
    throw new Error(
      `Invalid length for EnqueuePublicFunctionResult (got ${fields.length} expected ${ENQUEUE_PUBLIC_FUNCTION_CALL_RETURN_LENGTH})`,
    );
  }
  return fields.map(toACVMField);
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
