import { toHex } from './index.js';

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
});
