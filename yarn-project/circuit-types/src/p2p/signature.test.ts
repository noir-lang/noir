import { Fr } from '@aztec/foundation/fields';

import { recoverMessageAddress } from 'viem';

import { randomSigner } from './mocks.js';
import { Signature } from './signature.js';

describe('Signature serialization / deserialization', () => {
  it('Should serialize / deserialize', async () => {
    const signer = randomSigner();

    const originalMessage = Fr.random();
    const m = `0x${originalMessage.toBuffer().toString('hex')}`;

    const signature = await signer.signMessage({ message: m });

    const signatureObj = Signature.from0xString(signature);

    // Serde
    const serialized = signatureObj.toBuffer();
    const deserialized = Signature.fromBuffer(serialized);
    expect(deserialized).toEqual(signatureObj);

    const as0x = deserialized.to0xString();
    expect(as0x).toEqual(signature);

    // Recover signature
    const sender = await recoverMessageAddress({ message: originalMessage.toString(), signature: as0x });
    expect(sender).toEqual(signer.address);
  });
});
