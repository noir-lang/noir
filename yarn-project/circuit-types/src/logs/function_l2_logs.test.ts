import { EncryptedFunctionL2Logs, UnencryptedFunctionL2Logs } from './function_l2_logs.js';

function shouldBehaveLikeFunctionL2Logs(
  FunctionL2Logs: typeof UnencryptedFunctionL2Logs | typeof EncryptedFunctionL2Logs,
) {
  describe(FunctionL2Logs.name, () => {
    it('can encode L2Logs to buffer and back', () => {
      const l2Logs = FunctionL2Logs.random(3);

      const buffer = l2Logs.toBuffer();
      const recovered = FunctionL2Logs.fromBuffer(buffer);

      expect(recovered).toEqual(l2Logs);
    });

    it('can encode L2Logs to JSON and back', () => {
      const l2Logs = FunctionL2Logs.random(3);

      const buffer = l2Logs.toJSON();
      const recovered = FunctionL2Logs.fromJSON(buffer);

      expect(recovered).toEqual(l2Logs);
    });

    it('getSerializedLength returns the correct length', () => {
      const l2Logs = FunctionL2Logs.random(3);

      const buffer = l2Logs.toBuffer();
      const recovered = FunctionL2Logs.fromBuffer(buffer);

      expect(recovered.getSerializedLength()).toEqual(buffer.length);
    });

    it('getKernelLength returns the correct length', () => {
      const l2Logs = FunctionL2Logs.random(3);

      const expectedLength = l2Logs.logs.map(l => l.length).reduce((a, b) => a + b + 4, 0);

      expect(l2Logs.getKernelLength()).toEqual(expectedLength);
    });
  });
}

shouldBehaveLikeFunctionL2Logs(EncryptedFunctionL2Logs);
shouldBehaveLikeFunctionL2Logs(UnencryptedFunctionL2Logs);
