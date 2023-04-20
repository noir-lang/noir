import { pedersenCompressWithHashIndex } from '@aztec/barretenberg.js/crypto';
import { GeneratorIndex } from '@aztec/circuits.js';
import { AztecAddress, Fr, toBigInt } from '@aztec/foundation';
import { WasmWrapper } from '@aztec/foundation/wasm';

/**
 * Computes the index in the public data tree for a given contract and storage slot.
 * @param contract - address of the contract who owns the storage
 * @param slot - slot within the contract storage
 * @param bbWasm - wasm module for computing the hash
 * @returns The leaf index of the public data tree that maps to this storage slot
 */
export function computePublicDataTreeLeafIndex(contract: AztecAddress, slot: Fr, wasm: WasmWrapper): bigint {
  return toBigInt(
    pedersenCompressWithHashIndex(
      wasm,
      [contract, slot].map(f => f.toBuffer()),
      GeneratorIndex.PUBLIC_LEAF_INDEX,
    ),
  );
}
