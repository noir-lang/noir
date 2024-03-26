import { EncryptedTxL2Logs, UnencryptedTxL2Logs } from './tx_l2_logs.js';

function shouldBehaveLikeTxL2Logs(TxL2Logs: typeof EncryptedTxL2Logs | typeof UnencryptedTxL2Logs) {
  describe(TxL2Logs.name, () => {
    it('can encode TxL2Logs to buffer and back', () => {
      const l2Logs = TxL2Logs.random(6, 2);

      const buffer = l2Logs.toBuffer();
      const recovered = TxL2Logs.fromBuffer(buffer);

      expect(recovered).toEqual(l2Logs);
    });

    it('can encode TxL2Logs to JSON and back', () => {
      const l2Logs = TxL2Logs.random(6, 2);

      const buffer = l2Logs.toJSON();
      const recovered = TxL2Logs.fromJSON(buffer);

      expect(recovered).toEqual(l2Logs);
    });

    it('getSerializedLength returns the correct length', () => {
      const l2Logs = TxL2Logs.random(6, 2);

      const buffer = l2Logs.toBuffer();
      const recovered = TxL2Logs.fromBuffer(buffer);

      expect(recovered.getSerializedLength()).toEqual(buffer.length);
    });
  });
}

shouldBehaveLikeTxL2Logs(EncryptedTxL2Logs);
shouldBehaveLikeTxL2Logs(UnencryptedTxL2Logs);
