import { FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import {
  BufferReader,
  numToInt32BE,
  serializeBufferArrayToVector,
  serializeToBuffer,
} from '@aztec/foundation/serialize';
import { ContractClass } from '@aztec/types/contracts';

import chunk from 'lodash.chunk';

import { FUNCTION_SELECTOR_NUM_BYTES, MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS } from '../constants.gen.js';

/**
 * Packs together a set of public functions for a contract class.
 * @remarks This function should no longer be necessary once we have a single bytecode per contract.
 */
export function packBytecode(publicFns: ContractClass['publicFunctions']): Buffer {
  return serializeBufferArrayToVector(
    publicFns.map(fn => serializeToBuffer(fn.selector, fn.isInternal, numToInt32BE(fn.bytecode.length), fn.bytecode)),
  );
}

/**
 * Unpacks a set of public functions for a contract class from packed bytecode.
 * @remarks This function should no longer be necessary once we have a single bytecode per contract.
 */
export function unpackBytecode(buffer: Buffer): ContractClass['publicFunctions'] {
  const reader = BufferReader.asReader(buffer);
  return reader.readVector({
    fromBuffer: (reader: BufferReader) => ({
      selector: FunctionSelector.fromBuffer(reader.readBytes(FUNCTION_SELECTOR_NUM_BYTES)),
      isInternal: reader.readBoolean(),
      bytecode: reader.readBuffer(),
    }),
  });
}

/**
 * Formats packed bytecode as an array of fields. Splits the input into 31-byte chunks, and stores each
 * of them into a field, omitting the field's first byte, then adds zero-fields at the end until the max length.
 * @param packedBytecode - Packed bytecode for a contract.
 * @returns A field with the total length in bytes, followed by an array of fields such that their concatenation is equal to the input buffer.
 * @remarks This function is more generic than just for packed bytecode, perhaps it could be moved elsewhere.
 */
export function packedBytecodeAsFields(packedBytecode: Buffer): Fr[] {
  const encoded = [
    new Fr(packedBytecode.length),
    ...chunk(packedBytecode, Fr.SIZE_IN_BYTES - 1).map(c => {
      const fieldBytes = Buffer.alloc(32);
      Buffer.from(c).copy(fieldBytes, 1);
      return Fr.fromBuffer(fieldBytes);
    }),
  ];
  if (encoded.length > MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS) {
    throw new Error(
      `Packed bytecode exceeds maximum size: got ${encoded.length} but max is ${MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS}`,
    );
  }
  // Fun fact: we cannot use padArrayEnd here since typescript cannot deal with a Tuple this big
  return [...encoded, ...Array(MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS - encoded.length).fill(Fr.ZERO)];
}

/**
 * Recovers packed bytecode from an array of fields.
 * @param fields - An output from packedBytecodeAsFields.
 * @returns The packed bytecode.
 * @remarks This function is more generic than just for packed bytecode, perhaps it could be moved elsewhere.
 */
export function packedBytecodeFromFields(fields: Fr[]): Buffer {
  const [length, ...payload] = fields;
  return Buffer.concat(payload.map(f => f.toBuffer().subarray(1))).subarray(0, length.toNumber());
}
