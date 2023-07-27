import { PrivateKey } from './private_key.js';

describe('PrivateKey', () => {
  it('throws when the key buffer is not 32 bytes long', () => {
    expect(() => new PrivateKey(Buffer.alloc(0))).toThrowError(/Invalid private key length/);
  });

  it('can be created from a hex string', () => {
    const key = PrivateKey.fromString('0x' + 'a'.repeat(64));
    expect(key.value).toEqual(Buffer.from('a'.repeat(64), 'hex'));
  });

  it('correctly ignores 0x prefix in hex string', () => {
    const key = PrivateKey.fromString('0x' + 'a'.repeat(64));
    expect(key.value).toEqual(Buffer.from('a'.repeat(64), 'hex'));
  });
});
