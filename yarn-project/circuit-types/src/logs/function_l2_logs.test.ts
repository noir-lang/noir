import { EncryptedFunctionL2Logs, EncryptedNoteFunctionL2Logs, UnencryptedFunctionL2Logs } from './function_l2_logs.js';

function shouldBehaveLikeFunctionL2Logs(
  FunctionL2Logs:
    | typeof UnencryptedFunctionL2Logs
    | typeof EncryptedNoteFunctionL2Logs
    | typeof EncryptedFunctionL2Logs,
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
      if (FunctionL2Logs.name == 'EncryptedFunctionL2Logs') {
        // For event logs, we don't 'count' the maskedContractAddress as part of the
        // log length, since it's just for siloing later on
        expect(recovered.getSerializedLength()).toEqual(buffer.length - 3 * 32);
      } else {
        expect(recovered.getSerializedLength()).toEqual(buffer.length);
      }
    });

    it('getKernelLength returns the correct length', () => {
      const l2Logs = FunctionL2Logs.random(3);

      const expectedLength = l2Logs.logs.map(l => l.length).reduce((a, b) => a + b + 4, 0);

      expect(l2Logs.getKernelLength()).toEqual(expectedLength);
    });
  });
}

shouldBehaveLikeFunctionL2Logs(EncryptedNoteFunctionL2Logs);
shouldBehaveLikeFunctionL2Logs(UnencryptedFunctionL2Logs);
shouldBehaveLikeFunctionL2Logs(EncryptedFunctionL2Logs);
