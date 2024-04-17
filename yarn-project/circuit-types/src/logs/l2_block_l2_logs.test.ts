import { EncryptedL2BlockL2Logs, UnencryptedL2BlockL2Logs } from './l2_block_l2_logs.js';

function shouldBehaveLikeL2BlockL2Logs(L2BlockL2Logs: typeof EncryptedL2BlockL2Logs | typeof UnencryptedL2BlockL2Logs) {
  describe(L2BlockL2Logs.name, () => {
    it('can encode L2Logs to buffer and back', () => {
      const l2Logs = L2BlockL2Logs.random(3, 4, 2);

      const buffer = l2Logs.toBuffer();
      const recovered = L2BlockL2Logs.fromBuffer(buffer);

      expect(recovered).toEqual(l2Logs);
    });

    it('getSerializedLength returns the correct length', () => {
      const l2Logs = L2BlockL2Logs.random(3, 4, 2);

      const buffer = l2Logs.toBuffer();
      const recovered = L2BlockL2Logs.fromBuffer(buffer);

      expect(recovered.getSerializedLength()).toEqual(buffer.length);
    });

    it('serializes to and from JSON', () => {
      const l2Logs = L2BlockL2Logs.random(3, 4, 2);
      const json = l2Logs.toJSON();
      const recovered = L2BlockL2Logs.fromJSON(json);
      expect(recovered).toEqual(l2Logs);
    });
  });
}

shouldBehaveLikeL2BlockL2Logs(EncryptedL2BlockL2Logs);
shouldBehaveLikeL2BlockL2Logs(UnencryptedL2BlockL2Logs);
