import { CircuitsWasm, GeneratorIndex, GlobalVariables } from '@aztec/circuits.js';
import { computeGlobalsHash } from '@aztec/circuits.js/abis';
import { pedersenCompressWithHashIndex } from '@aztec/circuits.js/barretenberg';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { toBigInt } from '@aztec/foundation/serialize';
import { IWasmModule } from '@aztec/foundation/wasm';

/**
 * Computes the index in the public data tree for a given contract and storage slot.
 * @param contract - Address of the contract who owns the storage.
 * @param slot - Slot within the contract storage.
 * @param bbWasm - Wasm module for computing the hash.
 * @returns The leaf index of the public data tree that maps to this storage slot.
 */
export function computePublicDataTreeLeafIndex(contract: AztecAddress, slot: Fr, wasm: IWasmModule): bigint {
  return toBigInt(
    pedersenCompressWithHashIndex(
      wasm,
      [contract, slot].map(f => f.toBuffer()),
      GeneratorIndex.PUBLIC_LEAF_INDEX,
    ),
  );
}

/**
 * Computes the hash of the global variables.
 * @returns The hash of the global variables.
 */
export async function computeGlobalVariablesHash(
  globalVariables: GlobalVariables = GlobalVariables.empty(),
): Promise<Fr> {
  const wasm = await CircuitsWasm.get();
  return computeGlobalsHash(wasm, globalVariables);
}
