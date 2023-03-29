import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { randomBytes } from '@aztec/foundation/crypto';
import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';

describe('encrypt buffer', () => {
  it('convert to and from encrypted buffer', async () => {
    const grumpkin = new Grumpkin(await BarretenbergWasm.new());
    const data = randomBytes(253);
    const ownerPrivKey = randomBytes(32);
    const ownerPubKey = AztecAddress.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));
    const ephPrivKey = randomBytes(32);
    const encrypted = encryptBuffer(data, ownerPubKey, ephPrivKey, grumpkin);
    const decrypted = decryptBuffer(encrypted, ownerPrivKey, grumpkin);
    expect(decrypted).not.toBeUndefined();
    expect(decrypted).toEqual(data);
  });

  it('decrypting gibberish returns undefined', async () => {
    const grumpkin = new Grumpkin(await BarretenbergWasm.new());
    const data = randomBytes(253);
    const ownerPrivKey = randomBytes(32);
    const ephPrivKey = randomBytes(32);
    const ownerPubKey = AztecAddress.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));
    const encrypted = encryptBuffer(data, ownerPubKey, ephPrivKey, grumpkin);

    // Introduce gibberish.
    const gibberish = Buffer.concat([randomBytes(8), encrypted.subarray(8)]);

    const decrypted = decryptBuffer(gibberish, ownerPrivKey, grumpkin);
    expect(decrypted).toBeUndefined();
  });
});
