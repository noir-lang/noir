import { TextEncoder } from 'util';
import { Buffer128, Buffer32, Fq, Fr, Point } from '../types/index.js';
import { Barretenberg } from './index.js';
import { asyncMap } from '../async_map/index.js';

describe('schnorr', () => {
  const msg = Buffer.from(new TextEncoder().encode('The quick brown dog jumped over the lazy fox.'));
  let api: Barretenberg;

  beforeAll(async () => {
    api = await Barretenberg.new({ threads: 1 });
  }, 30000);

  afterAll(async () => {
    await api.destroy();
  });

  it('should verify signature', async () => {
    const pk = Fr.fromBuffer(
      new Uint8Array([
        0x0b, 0x9b, 0x3a, 0xde, 0xe6, 0xb3, 0xd8, 0x1b, 0x28, 0xa0, 0x88, 0x6b, 0x2a, 0x84, 0x15, 0xc7, 0xda, 0x31,
        0x29, 0x1a, 0x5e, 0x96, 0xbb, 0x7a, 0x56, 0x63, 0x9e, 0x17, 0x7d, 0x30, 0x1b, 0xeb,
      ]),
    );
    const pubKey = await api.schnorrComputePublicKey(pk);
    const [s, e] = await api.schnorrConstructSignature(msg, pk);
    const verified = await api.schnorrVerifySignature(msg, pubKey, s, e);

    expect(verified).toBe(true);
  });

  it('public key negation should work', async () => {
    const publicKeyStr =
      '0x164f01b1011a1b292217acf53eef4d74f625f6e9bd5edfdb74c56fd81aafeebb21912735f9266a3719f61c1eb747ddee0cac9917f5c807485d356709b529b62c';
    const publicKey = Point.fromString(publicKeyStr);
    // hardcoded expected negated public key
    const expectedInvertedStr =
      '0x164f01b1011a1b292217acf53eef4d74f625f6e9bd5edfdb74c56fd81aafeebb0ed3273ce80b35f29e5a2997ca397a6f1b874f3083f16948e6ac8e8a3ad649d5';
    const expectedInverted = Point.fromString(expectedInvertedStr);

    // negate - should match expected negated key
    const negatedPublicKey = await api.schnorrNegatePublicKey(publicKey);
    expect(negatedPublicKey.equals(expectedInverted)).toEqual(true);
    // negate again - should be original public key now
    expect((await api.schnorrNegatePublicKey(negatedPublicKey)).equals(publicKey)).toEqual(true);
  });

  it('should create + verify multi signature', async () => {
    // set up multisig accounts
    const numSigners = 7;
    const pks = [...Array(numSigners)].map(() => Fq.random());
    const pubKeys = await asyncMap(pks, pk => api.schnorrMultisigCreateMultisigPublicKey(pk));

    // round one
    const roundOnePublicOutputs: Buffer128[] = [];
    const roundOnePrivateOutputs: Buffer128[] = [];
    for (let i = 0; i < numSigners; ++i) {
      const [publicOutput, privateOutput] = await api.schnorrMultisigConstructSignatureRound1();
      roundOnePublicOutputs.push(publicOutput);
      roundOnePrivateOutputs.push(privateOutput);
    }

    // round two
    const roundTwoOutputs = await asyncMap(
      pks,
      async (pk, i) =>
        (
          await api.schnorrMultisigConstructSignatureRound2(
            msg,
            pk,
            roundOnePrivateOutputs[i],
            pubKeys,
            roundOnePublicOutputs,
          )
        )[0],
    );

    // generate signature
    const [s, e] = await api.schnorrMultisigCombineSignatures(msg, pubKeys, roundOnePublicOutputs, roundTwoOutputs)!;
    const [combinedKey] = await api.schnorrMultisigValidateAndCombineSignerPubkeys(pubKeys);
    expect(combinedKey).not.toEqual(Buffer.alloc(64));
    const verified = await api.schnorrVerifySignature(msg, combinedKey, s, e);
    expect(verified).toBe(true);
  });

  it('should identify invalid multi signature', async () => {
    const pks = [...Array(3)].map(() => Fq.random());
    const pubKeys = await asyncMap(pks, pk => api.schnorrMultisigCreateMultisigPublicKey(pk));
    const [combinedKey] = await api.schnorrMultisigValidateAndCombineSignerPubkeys(pubKeys);

    const verified = await api.schnorrVerifySignature(msg, combinedKey, Buffer32.random(), Buffer32.random());
    expect(verified).toBe(false);
  });

  it('should not construct invalid multi signature', async () => {
    // set up multisig accounts
    const numSigners = 7;
    const pks = [...Array(numSigners)].map(() => Fq.random());
    const pubKeys = await asyncMap(pks, pk => api.schnorrMultisigCreateMultisigPublicKey(pk));

    // round one
    const roundOnePublicOutputs: Buffer128[] = [];
    const roundOnePrivateOutputs: Buffer128[] = [];
    for (let i = 0; i < numSigners; ++i) {
      const [publicOutput, privateOutput] = await api.schnorrMultisigConstructSignatureRound1();
      roundOnePublicOutputs.push(publicOutput);
      roundOnePrivateOutputs.push(privateOutput);
    }

    // round two
    const roundTwoOutputs = await asyncMap(
      pks,
      async (pk, i) =>
        (
          await api.schnorrMultisigConstructSignatureRound2(
            msg,
            pk,
            roundOnePrivateOutputs[i],
            pubKeys,
            roundOnePublicOutputs,
          )
        )[0],
    );

    // wrong number of data
    {
      expect(
        (
          await api.schnorrMultisigCombineSignatures(
            msg,
            pubKeys.slice(0, -1),
            roundOnePublicOutputs.slice(0, -1),
            roundTwoOutputs.slice(0, -1),
          )
        )[2],
      ).toBe(false);
    }

    // invalid round two output
    {
      const invalidOutputs = [...roundTwoOutputs];
      invalidOutputs[1] = (
        await api.schnorrMultisigConstructSignatureRound2(
          msg,
          pks[2], // <- Wrong private key.
          roundOnePrivateOutputs[1],
          pubKeys,
          roundOnePublicOutputs,
        )
      )[0];
      expect((await api.schnorrMultisigCombineSignatures(msg, pubKeys, roundOnePublicOutputs, invalidOutputs))[2]).toBe(
        false,
      );
    }

    // contains duplicates
    {
      const invalidOutputs = [...roundTwoOutputs];
      invalidOutputs[1] = roundTwoOutputs[2];
      expect((await api.schnorrMultisigCombineSignatures(msg, pubKeys, roundOnePublicOutputs, invalidOutputs))[2]).toBe(
        false,
      );
    }
  });

  it('should not create combined key from public keys containing invalid key', async () => {
    const pks = [...Array(5)].map(() => Fq.random());
    const pubKeys = await asyncMap(pks, pk => api.schnorrMultisigCreateMultisigPublicKey(pk));

    // not a valid point
    {
      pubKeys[1] = new Buffer128(Buffer.alloc(128));
      expect((await api.schnorrMultisigValidateAndCombineSignerPubkeys(pubKeys))[1]).toBe(false);
    }

    // contains duplicates
    {
      pubKeys[1] = pubKeys[2];
      expect((await api.schnorrMultisigValidateAndCombineSignerPubkeys(pubKeys))[1]).toBe(false);
    }
  });
});
