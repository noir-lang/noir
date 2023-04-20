import { Grumpkin, pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { Fr, toBigIntBE } from '@aztec/foundation';
import { MAPPING_SLOT_PEDERSEN_CONSTANT } from './simulator.js';

export type NoirPoint = {
  x: bigint;
  y: bigint;
};

/**
 * Computes the resulting storage slot for an entry in a mapping.
 * @param mappingSlot - the slot of the mapping within state
 * @param owner - the key of the mapping
 * @param bbWasm - wasm module for computing
 * @returns The slot in the contract storage where the value is stored.
 */
export function computeSlotForMapping(mappingSlot: Fr, owner: NoirPoint | Fr, bbWasm: BarretenbergWasm) {
  const isFr = (owner: NoirPoint | Fr): owner is Fr => typeof (owner as Fr).value === 'bigint';
  const ownerField = isFr(owner) ? owner : new Fr(owner.x);

  return Fr.fromBuffer(
    pedersenCompressInputs(
      bbWasm,
      [MAPPING_SLOT_PEDERSEN_CONSTANT, mappingSlot, ownerField].map(f => f.toBuffer()),
    ),
  );
}

export function toPublicKey(privateKey: Buffer, grumpkin: Grumpkin): NoirPoint {
  const publicKey = grumpkin.mul(Grumpkin.generator, privateKey);
  return {
    x: toBigIntBE(publicKey.slice(0, 32)),
    y: toBigIntBE(publicKey.slice(32, 64)),
  };
}
