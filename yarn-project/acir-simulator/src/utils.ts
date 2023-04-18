import { Grumpkin, pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { Fr, toBigIntBE } from '@aztec/foundation';
import { MAPPING_SLOT_PEDERSEN_CONSTANT } from './simulator.js';

export type NoirPoint = {
  x: bigint;
  y: bigint;
};

export function computeSlot(mappingSlot: Fr, owner: NoirPoint | Fr, bbWasm: BarretenbergWasm) {
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
