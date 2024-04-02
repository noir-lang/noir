import { FunctionSelector } from '@aztec/foundation/abi';
import {
  BufferReader,
  numToInt32BE,
  serializeArrayOfBufferableToVector,
  serializeToBuffer,
} from '@aztec/foundation/serialize';
import { type ContractClass } from '@aztec/types/contracts';

import { FUNCTION_SELECTOR_NUM_BYTES } from '../constants.gen.js';

/**
 * Packs together a set of public functions for a contract class.
 * @remarks This function should no longer be necessary once we have a single bytecode per contract.
 */
export function packBytecode(publicFns: ContractClass['publicFunctions']): Buffer {
  return serializeArrayOfBufferableToVector(
    publicFns.map(fn => serializeToBuffer(fn.selector, numToInt32BE(fn.bytecode.length), fn.bytecode)),
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
      bytecode: reader.readBuffer(),
    }),
  });
}
