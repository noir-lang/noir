import { fromHex, toHex } from './index.js';

describe('bigint-buffer', () => {
  describe('toHex', () => {
    it('does not pad even length', () => {
      expect(toHex(16n)).toEqual('0x10');
    });

    it('pads odd length hex to even length', () => {
      expect(toHex(10n)).toEqual('0x0a');
    });

    it('pads zero to even length', () => {
      expect(toHex(0n)).toEqual('0x00');
    });

    it('pads zero to 32 bytes', () => {
      expect(toHex(0n, true)).toEqual('0x0000000000000000000000000000000000000000000000000000000000000000');
    });
  });

  describe('fromHex', () => {
    it('should convert a valid hex string to a Buffer', () => {
      const hexString = '0x1234567890abcdef';
      const expectedBuffer = Buffer.from('1234567890abcdef', 'hex');
      const result = fromHex(hexString);
      expect(result).toEqual(expectedBuffer);
    });

    it('should convert a valid hex string without prefix to a Buffer', () => {
      const hexString = '1234567890abcdef';
      const expectedBuffer = Buffer.from('1234567890abcdef', 'hex');
      const result = fromHex(hexString);
      expect(result).toEqual(expectedBuffer);
    });

    it('should throw an error for an invalid hex string', () => {
      const invalidHexString = '0x12345G';
      expect(() => fromHex(invalidHexString)).toThrow('Invalid hex string: 0x12345G');
    });

    it('should throw an error for an odd-length hex string', () => {
      const oddLengthHexString = '0x1234567';
      expect(() => fromHex(oddLengthHexString)).toThrow('Invalid hex string: 0x1234567');
    });

    it('should handle an empty hex string', () => {
      expect(fromHex('')).toEqual(Buffer.alloc(0));
      expect(fromHex('0x')).toEqual(Buffer.alloc(0));
    });
  });
});
